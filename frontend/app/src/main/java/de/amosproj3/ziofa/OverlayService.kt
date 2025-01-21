// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.app.Service
import android.content.Intent
import android.graphics.PixelFormat
import android.os.IBinder
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.ComposeView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.setViewTreeLifecycleOwner
import androidx.lifecycle.setViewTreeViewModelStoreOwner
import androidx.savedstate.setViewTreeSavedStateRegistryOwner
import de.amosproj3.ziofa.shared.OVERLAY_POSITION_EXTRA
import de.amosproj3.ziofa.ui.overlay.OverlayLifecycleOwner
import de.amosproj3.ziofa.ui.overlay.OverlayRoot
import de.amosproj3.ziofa.ui.visualization.data.OverlayPosition
import timber.log.Timber

class OverlayService : Service() {

    class OverlayViewModelStoreOwner : ViewModelStoreOwner {
        override val viewModelStore: ViewModelStore =
            ViewModelStore().also { Timber.i("created viewmodel store $it") }
    }

    private val layoutParams =
        WindowManager.LayoutParams(
            /* w = */ WindowManager.LayoutParams.WRAP_CONTENT,
            /* h = */ WindowManager.LayoutParams.WRAP_CONTENT,
            /* _type = */ WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            /* _flags = */ WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE or
                WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or
                WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE,
            /* _format = */ PixelFormat.TRANSLUCENT,
        )

    private val windowManager by lazy { getSystemService(WINDOW_SERVICE) as WindowManager }
    private val lifecycleOwner by lazy { OverlayLifecycleOwner() }

    private var activeOverlay: View? = null

    override fun onCreate() {
        Timber.i("onCreate()")
        super.onCreate()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Timber.i("onStartCommand")
        setTheme(R.style.Theme_ZIOFA)
        layoutParams.applyOverlayPositionAsGravity(intent)
        showOverlay()
        return super.onStartCommand(intent, flags, startId)
    }

    private fun showOverlay() {
        val composeView = createComposeViewWithLifecycle { OverlayRoot() }
        teardownOverlay() // Make sure only one overlay window is active at a time
        setupOverlay(composeView, layoutParams)
    }

    override fun onDestroy() {
        super.onDestroy()
        Timber.i("onDestroy()")
        lifecycleOwner.handleLifecycleEvent(Lifecycle.Event.ON_DESTROY)
        activeOverlay?.let { windowManager.removeView(it) }
    }

    override fun onBind(p0: Intent?): IBinder? {
        return null
    }

    private fun setupOverlay(view: View, params: WindowManager.LayoutParams) {
        windowManager.addView(view, params)
        activeOverlay = view
        lifecycleOwner.handleLifecycleEvent(Lifecycle.Event.ON_START)
    }

    private fun teardownOverlay() {
        activeOverlay?.let { windowManager.removeView(it) }
        activeOverlay = null
    }

    private fun createComposeViewWithLifecycle(content: @Composable () -> Unit): ComposeView {
        val composeView = ComposeView(this)
        lifecycleOwner.attach()
        lifecycleOwner.performRestore(null)
        lifecycleOwner.handleLifecycleEvent(Lifecycle.Event.ON_CREATE)
        composeView.setViewTreeLifecycleOwner(lifecycleOwner)
        composeView.setViewTreeViewModelStoreOwner(OverlayViewModelStoreOwner())
        composeView.setViewTreeSavedStateRegistryOwner(lifecycleOwner)

        composeView.setContent { content() }
        return composeView
    }

    private fun WindowManager.LayoutParams.applyOverlayPositionAsGravity(intent: Intent?) {
        val overlayPosition =
            intent?.getStringExtra(OVERLAY_POSITION_EXTRA)?.let { OverlayPosition.valueOf(it) }
        this.apply {
            gravity =
                when (overlayPosition) {
                    OverlayPosition.BottomLeft -> Gravity.START or Gravity.BOTTOM
                    OverlayPosition.TopLeft -> Gravity.START or Gravity.TOP
                    OverlayPosition.BottomRight -> Gravity.END or Gravity.BOTTOM
                    OverlayPosition.TopRight -> Gravity.END or Gravity.TOP
                    null -> Gravity.START or Gravity.BOTTOM
                }
        }
    }
}
