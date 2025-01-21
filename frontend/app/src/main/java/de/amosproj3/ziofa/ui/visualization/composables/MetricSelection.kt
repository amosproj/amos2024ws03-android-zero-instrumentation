// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.SelectionData

@Composable
fun MetricSelection(
    selectionData: SelectionData,
    optionSelected: (DropdownOption) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(modifier.fillMaxWidth()) {
        val dropdownModifier = Modifier.weight(1f).padding(end = 0.dp)

        MetricDropdown(
            selectionData.componentOptions,
            "Select a package",
            modifier = dropdownModifier,
            optionSelected = { optionSelected(it) },
            selectedOption = selectionData.selectedComponent.displayName,
        )
        selectionData.metricOptions
            ?.takeIf { it.isNotEmpty() }
            ?.let { metricOptions ->
                MetricDropdown(
                    metricOptions,
                    "Select a metric",
                    modifier = dropdownModifier,
                    optionSelected = { optionSelected(it) },
                    selectedOption = selectionData.selectedMetric?.displayName ?: "Please select...",
                )
            } ?: Spacer(Modifier.weight(1f))
        selectionData.timeframeOptions
            ?.takeIf { it.isNotEmpty() }
            ?.let { timeframeOptions ->
                MetricDropdown(
                    timeframeOptions,
                    "Select an interval for aggregation",
                    modifier = dropdownModifier,
                    optionSelected = { optionSelected(it) },
                    selectedOption =
                        selectionData.selectedTimeframe?.displayName ?: "Please select...",
                )
            } ?: Spacer(Modifier.weight(1f))
    }
}
