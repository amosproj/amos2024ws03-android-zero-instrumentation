// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.overlay

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.overlay.OverlayController
import de.amosproj3.ziofa.api.overlay.OverlayState
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.mappings.getChartData
import de.amosproj3.ziofa.ui.visualization.mappings.getChartMetadata
import de.amosproj3.ziofa.ui.visualization.utils.isValidSelection
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn

@OptIn(ExperimentalCoroutinesApi::class)
class OverlayViewModel(
    val overlayManager: OverlayController,
    val dataStreamProviderFactory: (CoroutineScope) -> DataStreamProvider,
) : ViewModel() {

    private val dataStreamProvider = dataStreamProviderFactory(viewModelScope)

    val overlaySettings =
        overlayManager.overlayState
            .map { it.overlaySettings }
            .stateIn(viewModelScope, SharingStarted.Lazily, OverlaySettings())

    val overlayData =
        overlayManager.overlayState
            .mapNotNull { it as? OverlayState.Enabled }
            .mapNotNull { it.selectionData }
            .flatMapLatest {
                if (
                    isValidSelection(it.selectedComponent, it.selectedMetric, it.selectedTimeframe)
                ) {
                    dataStreamProvider.getChartData(
                        it.selectedComponent,
                        it.selectedMetric,
                        it.selectedTimeframe,
                        chartMetadata = it.selectedMetric.getChartMetadata(),
                    ) ?: flowOf(GraphedData.EMPTY)
                } else flowOf(GraphedData.EMPTY)
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, GraphedData.EMPTY)
}
