// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.selection

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.Icon
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.google.accompanist.drawablepainter.rememberDrawablePainter
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlinx.collections.immutable.ImmutableList

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MetricDropdown(
    options: ImmutableList<DropdownOption>,
    title: String,
    modifier: Modifier = Modifier,
    optionSelected: (DropdownOption) -> Unit,
    selectedOption: String?,
) {
    var expanded by remember { mutableStateOf(false) }

    Box(modifier = modifier) {
        ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
            TextField(
                value = selectedOption?: "Please select ...",
                onValueChange = {},
                readOnly = true,
                label = { Text(title) },
                trailingIcon = { Icon(Icons.Default.ArrowDropDown, contentDescription = null) },
                modifier = Modifier.menuAnchor(MenuAnchorType.PrimaryNotEditable).fillMaxWidth(),
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
                            if (option is DropdownOption.App) {
                                val painter = rememberDrawablePainter(option.icon)
                                Image(
                                    painter = painter,
                                    contentDescription = "",
                                    modifier = Modifier.size(50.dp, 50.dp),
                                )
                            } else if (option is DropdownOption.Process) {
                                Image(
                                    imageVector = Icons.Filled.Info,
                                    contentDescription = "",
                                    modifier = Modifier.size(50.dp, 50.dp),
                                )
                            }
                        },
                        onClick = {
                            optionSelected(option)
                            expanded = false
                        },
                    )
                }
            }
        }
    }
}
