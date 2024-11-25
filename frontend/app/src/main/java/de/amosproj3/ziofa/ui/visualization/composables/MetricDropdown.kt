// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

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
import com.google.accompanist.drawablepainter.rememberDrawablePainter
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MetricDropdown(
    options: List<DropdownOption>, // TODO replace with data class
    title: String,
    modifier: Modifier = Modifier,
    optionSelected: (DropdownOption) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    var selected by remember { mutableStateOf(options[0].displayName) }

    Box(modifier = modifier) {
        ExposedDropdownMenuBox(
            expanded = expanded,
            onExpandedChange = { expanded = it },
            modifier = modifier,
        ) {
            TextField(
                value = selected,
                onValueChange = {},
                readOnly = true,
                label = { Text(title) },
                trailingIcon = { Icon(Icons.Default.ArrowDropDown, contentDescription = null) },
                modifier = Modifier.menuAnchor().fillMaxWidth(),
            )
            ExposedDropdownMenu(
                modifier = modifier,
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                options.forEach { option ->
                    DropdownMenuItem(
                        text = { Text(option.displayName) },
                        trailingIcon = {
                            if (option is DropdownOption.AppOption) {
                                val painter = rememberDrawablePainter(option.icon)
                                Icon(painter = painter, contentDescription = "")
                            }
                        },
                        onClick = {
                            optionSelected(option)
                            selected = option.displayName
                            expanded = false
                        },
                    )
                }
            }
        }
    }
}
