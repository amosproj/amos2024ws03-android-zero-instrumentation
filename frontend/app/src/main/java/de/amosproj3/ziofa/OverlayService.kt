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

/**
 * This service is required by the overlay for starting, stopping it and feeding it data. The
 * lifecycle of the ViewModels needs to be managed manually via the [lifecycleOwner] Currently, the
 * lifecycle of the overlay viewmodels is tied to the lifecycle of the service.
 */
class OverlayService : Service() {

    /** Required to manage the view models. * */
    class OverlayViewModelStoreOwner : ViewModelStoreOwner {
        override val viewModelStore: ViewModelStore =
            ViewModelStore().also { Timber.i("created viewmodel store $it") }
    }

    /**
     * The layout parameters for the overlay window. The position should a added later so we can
     * change it. Currently, the overlay is touch-through and half-transparent.
     */
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

    /** It is important that this is only initialized once and not created multiple times. */
    private val lifecycleOwner by lazy { OverlayLifecycleOwner() }

    /** Keep the active View for tearing it down upon stopping the service */
    private var activeOverlay: View? = null

    override fun onCreate() {
        Timber.i("onCreate()")
        super.onCreate()
    }

    /**
     * When the service is started, the overlay will be rendered. By getting the extra from the
     * intent, we set the position of the overlay.
     */
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Timber.i("onStartCommand")
        setTheme(R.style.Theme_ZIOFA)
        layoutParams.applyOverlayPositionAsGravity(intent)
        showOverlay()
        return super.onStartCommand(intent, flags, startId)
    }

    /**
     * First tear down any active overlays to prevent overlapping, then display the overlay with the
     * [layoutParams].
     */
    private fun showOverlay() {
        val composeView = createComposeViewWithLifecycle { OverlayRoot() }
        teardownOverlay() // Make sure only one overlay window is active at a time
        setupOverlay(composeView, layoutParams)
    }

    /**
     * The service is stopped, we have to pass that lifecycle event to the [lifecycleOwner] as well
     * and remove the active window.
     */
    override fun onDestroy() {
        super.onDestroy()
        Timber.i("onDestroy()")
        lifecycleOwner.handleLifecycleEvent(Lifecycle.Event.ON_DESTROY)
        teardownOverlay()
    }

    /** We don't need to bind to the service. */
    override fun onBind(p0: Intent?): IBinder? {
        return null
    }

    /** Add the overlay view as a window and pass the lifecycle event to the [lifecycleOwner] */
    private fun setupOverlay(view: View, params: WindowManager.LayoutParams) {
        windowManager.addView(view, params)
        activeOverlay = view
        lifecycleOwner.handleLifecycleEvent(Lifecycle.Event.ON_START)
    }

    /** Remove the active overlay window */
    private fun teardownOverlay() {
        activeOverlay?.let { windowManager.removeView(it) }
        activeOverlay = null
    }

    /**
     * Very important, the [lifecycleOwner] needs to be attached and the [Lifecycle.Event.ON_CREATE]
     * passed, otherwise the viewmodel will never be instantiated properly.
     */
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

    /**
     * Convert the overlay position passed via the intent into a gravity value for the layout params
     * of the window.
     */
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
