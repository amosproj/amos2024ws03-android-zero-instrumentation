// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

enum class VisualizationType {
    GAUGE_OVER_TIME,
    HISTOGRAM,
    COUNTER,
    DISTRIBUTION,
    LOG,
}

data class VisualizationUIInfo(
    val featureName: String,
    val visualizationType: VisualizationType,
    val isGlobal: Boolean,
    val hasTimeframe: Boolean,
)

object BackendToUiMapping {
    private val backendToUiVisualization =
        mapOf(
            "xdp_counter" to
                {
                    VisualizationUIInfo(
                        "Packages per X",
                        visualizationType = VisualizationType.GAUGE_OVER_TIME,
                        isGlobal = true,
                        hasTimeframe = true,
                    )
                },
            "vfs_write" to
                {
                    VisualizationUIInfo(
                        featureName = "VFS Write",
                        visualizationType = VisualizationType.HISTOGRAM,
                        isGlobal = true,
                        hasTimeframe = false,
                    )
                },
        )

    fun getVisualization(backendName: String) = backendToUiVisualization[backendName]
}
