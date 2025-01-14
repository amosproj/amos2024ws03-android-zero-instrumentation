// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import android.graphics.drawable.Drawable
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import kotlin.time.DurationUnit

/** These all need to be of the same type or else we need seperate dropdown composables */
sealed class DropdownOption(val displayName: String) {

    /** Filter options */
    data class Process(val processName: String, val pid: UInt) : DropdownOption(processName)

    data class App(
        val appName: String,
        val packageName: String,
        val icon: Drawable,
        val pids: List<UInt>,
    ) : DropdownOption(appName)

    data object Global : DropdownOption("Global")

    /**
     * Metric options
     *
     * @param backendFeature the associated backend feature of the metric option
     */
    data class Metric(val backendFeature: BackendFeatureOptions) :
        DropdownOption(backendFeature.featureName)

    /** Timeframe options */
    data class Timeframe(val amount: Int, val unit: DurationUnit) : DropdownOption("$amount $unit")
}

data class SelectionData(
    val componentOptions: List<DropdownOption>,
    val metricOptions: List<DropdownOption>?,
    val timeframeOptions: List<DropdownOption>?,
    val selectedComponent: DropdownOption,
    val selectedMetric: DropdownOption?,
    val selectedTimeframe: DropdownOption?,
)
