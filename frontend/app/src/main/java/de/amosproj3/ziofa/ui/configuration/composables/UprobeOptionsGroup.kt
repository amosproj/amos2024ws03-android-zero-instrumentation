// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.FeatureType
import kotlinx.collections.immutable.ImmutableList

@Composable
fun EbpfUprobeFeatureOptions(
    options: ImmutableList<BackendFeatureOptions.UprobeOption>,
    onOptionDeleted: (BackendFeatureOptions.UprobeOption) -> Unit,
    onAddUprobeSelected: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(horizontal = 20.dp, vertical = 15.dp).fillMaxSize()) {
        SectionTitleRow(FeatureType.UPROBES.displayName)
        Spacer(Modifier.height(15.dp))
        options.forEach { option ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.name)
                IconButton(onClick = { onOptionDeleted(option) }) {
                    Icon(Icons.Default.Delete, contentDescription = "")
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Add new uprobe ...")
            IconButton(onClick = { onAddUprobeSelected() }) {
                Icon(Icons.Default.Add, contentDescription = "")
            }
        }
    }
}
