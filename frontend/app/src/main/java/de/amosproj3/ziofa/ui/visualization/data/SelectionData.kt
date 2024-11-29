// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import android.graphics.drawable.Drawable
import kotlin.time.DurationUnit

/** These all need to be of the same type or else we need seperate dropdown composables */
sealed class DropdownOption(val displayName: String) {

    /** Filter options */
    data class Process(val processName: String, val pid: UInt) : DropdownOption(processName)

    data class AppOption(
        val appName: String,
        val packageName: String,
        val icon: Drawable,
        val pids: List<UInt>,
    ) : DropdownOption(appName)

    data object Global : DropdownOption("Global")

    /**
     * Metric options
     *
     * @param metricName the displayed name
     * @param ebpfName the "ID"
     */
    data class MetricOption(val metricName: String) : DropdownOption(metricName)

    /** Timeframe options */
    data class TimeframeOption(val amount: Int, val unit: DurationUnit) :
        DropdownOption("$amount $unit")
}

data class SelectionData(
    val filterOptions: List<DropdownOption>,
    val metricOptions: List<DropdownOption>?,
    val timeframeOptions: List<DropdownOption>?,
    val selectedFilter: DropdownOption,
    val selectedMetric: DropdownOption?,
    val selectedTimeframe: DropdownOption?,
)
