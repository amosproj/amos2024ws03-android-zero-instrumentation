// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.SectionTitleRow
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

@Composable
fun EbpfIOFeatureOptions(
    options: List<BackendFeatureOptions>,
    onOptionChanged: (BackendFeatureOptions, Boolean) -> Unit,
) {
    LazyColumn(modifier = Modifier.padding(horizontal = 20.dp, vertical = 15.dp).fillMaxWidth()) {
        items(options) { option ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.featureName)
                Checkbox(checked = option.active, onCheckedChange = { onOptionChanged(option, it) })
            }
        }
    }
}
