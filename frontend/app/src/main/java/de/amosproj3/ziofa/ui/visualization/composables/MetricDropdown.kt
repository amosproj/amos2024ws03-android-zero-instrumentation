// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MetricDropdown(
    options: List<Pair<String, ImageVector?>>, // TODO replace with data class
    title: String,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }
    var selected by remember { mutableStateOf(options[0].first) }

    Box(modifier = modifier) {
        ExposedDropdownMenuBox(
            expanded = expanded,
            onExpandedChange = { !expanded },
            modifier = modifier,
        ) {
            TextField(
                value = selected,
                onValueChange = {},
                readOnly = true,
                label = { Text(title) },
                trailingIcon = { Icon(Icons.Default.ArrowDropDown, contentDescription = null) },
                modifier = Modifier.clickable { expanded = !expanded }.fillMaxWidth(),
            )
            ExposedDropdownMenu(
                modifier = modifier,
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                options.forEach { (displayName, icon) ->
                    DropdownMenuItem(
                        text = { Text(displayName) },
                        trailingIcon = {
                            icon?.let { Icon(imageVector = it, contentDescription = "") }
                        },
                        onClick = {},
                    )
                }
            }
        }
    }
}
