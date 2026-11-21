// MyLock 生物识别插件（自定义 Tauri mobile plugin）。
//
// 安全边界（见 android-feasibility.md §3 / Mnemon 文档 88709829）：
// - Android Keystore 里只放一把 AES-GCM 密钥 K_dev，且 setUserAuthenticationRequired(true)，
//   必须经 BiometricPrompt(CRYPTO_OBJECT) 认证后才能使用；它只在本机 wrap「口令字节副本」。
// - K_dev 绝不当跨端主密钥；解出的口令字节经 run_mobile_plugin 回 Rust，再走 full_open。
// - Kotlin 层不实现任何密码学算法：AES-GCM/随机数全用平台 Keystore/JCE；Base64 只是编码。
// - 口令兜底永远可用（Rust 侧 unlock_vault 仍接受主口令/恢复码）。

package app.tauri.biometric

import android.app.Activity
import android.content.Context
import androidx.fragment.app.FragmentActivity
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val KEY_ALIAS = "mylock_bio_wrap_v1"
private const val PREFS_NAME = "mylock_bio_wrap"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val TAG_LENGTH_BITS = 128
private const val IV_BYTES = 12

@InvokeArg
class EnrollArgs {
    lateinit var root: String
    lateinit var passwordB64: String
}

@InvokeArg
class UnlockArgs {
    lateinit var root: String
}

@TauriPlugin
class BiometricPlugin(private val activity: Activity) : Plugin(activity) {
    private val executor = ContextCompat.getMainExecutor(activity)
    private val prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    /** SharedPreferences 的 key 直接由 root 绝对路径派生（不落明文口令）。 */
    private fun slot(root: String): String = "wrap_" + root

    private fun keystore(): KeyStore =
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    private fun ensureKey() {
        val ks = keystore()
        if (!ks.containsAlias(KEY_ALIAS)) {
            val spec = KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .setUserAuthenticationRequired(true)
                // 增删指纹后旧密文不可解（Java AEADBadTagException），UI 已提示重新启用。
                .setInvalidatedByBiometricEnrollment(true)
                .build()
            val kg = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
            kg.init(spec)
            kg.generateKey()
        }
    }

    private fun key(): SecretKey = keystore().getKey(KEY_ALIAS, null) as SecretKey

    private fun biometricProblem(): String? = when (
        BiometricManager.from(activity)
            .canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG)
    ) {
        BiometricManager.BIOMETRIC_SUCCESS -> null
        BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE -> "本机无生物识别硬件，请用口令解锁。"
        BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE -> "生物识别硬件暂不可用，请用口令解锁。"
        BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> "尚未录入指纹/面部，请到系统设置录入后再启用。"
        BiometricManager.BIOMETRIC_ERROR_SECURITY_UPDATE_REQUIRED -> "需要安全更新后才能使用生物识别。"
        BiometricManager.BIOMETRIC_ERROR_UNSUPPORTED -> "系统不支持 Class 3 生物识别，请用口令解锁。"
        else -> "生物识别不可用，请用口令解锁。"
    }

    private fun promptInfo(): BiometricPrompt.PromptInfo =
        BiometricPrompt.PromptInfo.Builder()
            .setTitle("MyLock 身份验证")
            .setSubtitle("验证指纹或面部以继续")
            .setNegativeButtonText("取消")
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
            .build()

    /**
     * 用 CryptoObject 包住已初始化的 cipher，认证成功后 cipher 才可用于加/解密。
     * BiometricPrompt 构造器要求 FragmentActivity；TauriActivity 继承 AppCompatActivity
     * （属于 FragmentActivity），运行期必然可转型。
     */
    private fun authenticate(invoke: Invoke, cipher: Cipher, onSuccess: (Cipher) -> Unit) {
        val fragmentActivity = activity as? FragmentActivity
            ?: run {
                invoke.reject("宿主 Activity 不是 FragmentActivity，无法使用生物识别。")
                return
            }
        val prompt = BiometricPrompt(
            fragmentActivity,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    val authCipher = result.cryptoObject?.cipher
                    if (authCipher == null) {
                        invoke.reject("生物识别成功但未取得加密对象。")
                        return
                    }
                    try {
                        onSuccess(authCipher)
                    } catch (e: Exception) {
                        invoke.reject("操作失败：${e.message}")
                    }
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    invoke.reject(
                        if (errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON) {
                            "已取消。"
                        } else {
                            "生物识别失败：$errString"
                        }
                    )
                }

                override fun onAuthenticationFailed() {
                    // 单次不匹配不结束，交给系统提示重试。
                }
            }
        )
        prompt.authenticate(promptInfo(), BiometricPrompt.CryptoObject(cipher))
    }

    /** 首次启用：认证后用 K_dev 加密口令字节，密文写入 SharedPreferences。 */
    @Command
    fun enroll(invoke: Invoke) {
        val args = invoke.parseArgs(EnrollArgs::class.java)
        biometricProblem()?.let { invoke.reject(it); return }
        try {
            val password = B64.decode(args.passwordB64)
            if (password.isEmpty()) {
                invoke.reject("口令数据无效。")
                return
            }
            ensureKey()
            val cipher = Cipher.getInstance(TRANSFORMATION)
            // AndroidKeyStore 的 setUserAuthenticationRequired(true) 密钥禁止调用方自带 IV
            // （ENCRYPT_MODE 下会抛 "Caller-provided IV not permitted"）。必须由 Keystore
            // 自己生成 IV，init 后用 cipher.iv 读回并与密文一起保存。
            cipher.init(Cipher.ENCRYPT_MODE, key())
            val iv = cipher.iv
            authenticate(invoke, cipher) { c ->
                val ct = c.doFinal(password)
                prefs.edit()
                    .putString(slot(args.root), B64.encode(iv) + "." + B64.encode(ct))
                    .commit()
                invoke.resolve()
            }
        } catch (e: Exception) {
            invoke.reject("启用生物识别失败：${e.message}")
        }
    }

    /** 解锁：认证后解出口令字节，base64 回传 Rust（BioSecret.password）走 full_open。 */
    @Command
    fun unlock(invoke: Invoke) {
        val args = invoke.parseArgs(UnlockArgs::class.java)
        biometricProblem()?.let { invoke.reject(it); return }
        val blob = prefs.getString(slot(args.root), null)
        if (blob == null) {
            invoke.reject("该保险柜尚未启用生物识别，请用主口令解锁。")
            return
        }
        try {
            val parts = blob.split(".")
            if (parts.size != 2) {
                invoke.reject("生物识别数据损坏，请用口令解锁后重新启用。")
                return
            }
            val iv = B64.decode(parts[0])
            val ct = B64.decode(parts[1])
            if (iv.size != IV_BYTES || ct.isEmpty()) {
                invoke.reject("生物识别数据无效，请用口令解锁后重新启用。")
                return
            }
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_LENGTH_BITS, iv))
            authenticate(invoke, cipher) { c ->
                val password = c.doFinal(ct)
                val result = JSObject()
                result.put("password", B64.encode(password))
                invoke.resolve(result)
            }
        } catch (e: Exception) {
            invoke.reject("解密失败（生物识别信息可能已变更）：${e.message}")
        }
    }
}

/** 纯编码工具（与 Rust 侧 base64 实现一致），不涉及密码学。 */
private object B64 {
    private const val A = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"

    fun encode(bytes: ByteArray): String {
        val sb = StringBuilder()
        var i = 0
        while (i < bytes.size) {
            val b0 = bytes[i].toInt() and 0xff
            val b1 = if (i + 1 < bytes.size) bytes[i + 1].toInt() and 0xff else 0
            val b2 = if (i + 2 < bytes.size) bytes[i + 2].toInt() and 0xff else 0
            val n = (b0 shl 16) or (b1 shl 8) or b2
            sb.append(A[(n shr 18) and 0x3f])
            sb.append(A[(n shr 12) and 0x3f])
            sb.append(if (i + 1 < bytes.size) A[(n shr 6) and 0x3f] else '=')
            sb.append(if (i + 2 < bytes.size) A[n and 0x3f] else '=')
            i += 3
        }
        return sb.toString()
    }

    fun decode(s: String): ByteArray {
        val clean = s.filter { it != '=' && !it.isWhitespace() }
        require(clean.length % 4 != 1) { "非法 base64 长度" }
        val out = ByteArrayOutputStream()
        var i = 0
        while (i < clean.length) {
            val n = minOf(4, clean.length - i)
            var acc = 0
            for (j in 0 until 4) {
                acc = acc shl 6
                if (j < n) {
                    val idx = A.indexOf(clean[i + j])
                    require(idx >= 0) { "非法 base64 字符" }
                    acc = acc or idx
                }
            }
            out.write((acc shr 16) and 0xff)
            if (n > 2) out.write((acc shr 8) and 0xff)
            if (n > 3) out.write(acc and 0xff)
            i += 4
        }
        return out.toByteArray()
    }
}
