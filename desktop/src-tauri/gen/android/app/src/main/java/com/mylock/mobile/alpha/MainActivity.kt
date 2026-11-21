package com.mylock.mobile.alpha

import android.os.Bundle
import android.view.WindowManager
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // 封闭内测：禁止系统截屏/录屏/最近任务缩略图泄露明文暗格内容。
    window.setFlags(
      WindowManager.LayoutParams.FLAG_SECURE,
      WindowManager.LayoutParams.FLAG_SECURE
    )
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
