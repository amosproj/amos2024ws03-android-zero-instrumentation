// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import androidx.compose.ui.graphics.vector.ImageVector
import java.util.concurrent.TimeUnit

data class PackageOption(val packageName: String, val displayName: String?, val logo: ImageVector?)

data class MetricOption(val displayName: String)

data class TimeframeOption(val amount: Int, val unit: TimeUnit)

class SelectionData(
    val packageOptions: List<PackageOption>,
    val metricOptions: List<MetricOption>,
    val timeframeOptions: List<TimeframeOption>,
)
