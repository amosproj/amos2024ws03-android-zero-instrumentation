// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.platform.overlay

import android.content.Context
import android.content.Intent
import com.freeletics.flowredux.dsl.FlowReduxStateMachine
import de.amosproj3.ziofa.OverlayService
import de.amosproj3.ziofa.api.overlay.OverlayAction
import de.amosproj3.ziofa.api.overlay.OverlayController
import de.amosproj3.ziofa.api.overlay.OverlayState
import de.amosproj3.ziofa.shared.OVERLAY_POSITION_EXTRA
import de.amosproj3.ziofa.ui.visualization.data.OverlayPosition
import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber

private val INITIAL_OVERLAY_SETTINGS = OverlaySettings()
val INITIAL_OVERLAY_STATE = OverlayState.Disabled(INITIAL_OVERLAY_SETTINGS)

/** Communication layer between service and UI */
@OptIn(ExperimentalCoroutinesApi::class)
class OverlayManager(private val context: Context) :
    FlowReduxStateMachine<OverlayState, OverlayAction>(initialState = INITIAL_OVERLAY_STATE),
    OverlayController {

    override val overlayState = MutableStateFlow<OverlayState>(INITIAL_OVERLAY_STATE)

    override fun dispatchAction(action: OverlayAction) {
        CoroutineScope(Dispatchers.IO).launch { this@OverlayManager.dispatch(action) }
    }

    init {
        initializeStateMachine()
        startUpdatingState()
    }

    private fun initializeStateMachine() {
        spec {
            inState<OverlayState.Disabled> {
                onEnter {
                    stopOverlayService()
                    it.noChange()
                }
                on<OverlayAction.ChangeSettings> { action, state ->
                    state.mutate { this.copy(overlaySettings = action.newSettings) }
                }
                on<OverlayAction.Enable> { action, state ->
                    state.override {
                        OverlayState.Enabled(overlaySettings, selectionData = action.selectionData)
                    }
                }
            }
            inState<OverlayState.Enabled> {
                onEnter {
                    startOverlayService(it.snapshot.overlaySettings.selectedPosition)
                    it.noChange()
                }
                on<OverlayAction.ChangeSettings> { action, state ->
                    state.mutate { copy(overlaySettings = action.newSettings) }
                }
                on<OverlayAction.Enable> { action, state ->
                    state.mutate { copy(selectionData = action.selectionData) }
                }
                on<OverlayAction.Disable> { _, state ->
                    state.override { OverlayState.Disabled(overlaySettings) }
                }
            }
        }
    }

    private fun startUpdatingState() {
        CoroutineScope(Dispatchers.IO).launch {
            this@OverlayManager.state.collect {
                Timber.i("overlay state updated $it")
                overlayState.value = it
            }
        }
    }

    private fun startOverlayService(overlayPosition: OverlayPosition) {
        context.startService(
            Intent(context, OverlayService::class.java).apply {
                putExtra(OVERLAY_POSITION_EXTRA, overlayPosition.name)
            }
        )
    }

    private fun stopOverlayService() {
        context.stopService(Intent(context, OverlayService::class.java))
    }
}
