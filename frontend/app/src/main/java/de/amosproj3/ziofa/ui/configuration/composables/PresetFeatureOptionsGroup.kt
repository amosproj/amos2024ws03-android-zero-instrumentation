// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.composables

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.FeatureType
import kotlinx.collections.immutable.ImmutableList

/**
 * Displays a group of options along with a title. All [options] must be of the specified [type] and
 * only those will be shown.
 */
@Composable
fun PresetFeatureOptionsGroup(
    options: ImmutableList<BackendFeatureOptions>,
    type: FeatureType,
    onOptionChanged: (BackendFeatureOptions, Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(horizontal = 20.dp, vertical = 15.dp).fillMaxWidth()) {
        SectionTitleRow(type.displayName)
        Spacer(Modifier.height(15.dp))
        options
            .filter { it.type == type }
            .forEach { option ->
                Row(
                    modifier = Modifier.fillMaxWidth().padding(bottom = 15.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Image(
                            imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                            contentDescription = "",
                            modifier = Modifier.padding(end = 10.dp),
                        )
                        Column {
                            Text(option.name, fontWeight = FontWeight.Bold)
                            Text(option.description, fontStyle = FontStyle.Italic)
                        }
                    }
                    Checkbox(
                        checked = option.active,
                        onCheckedChange = { onOptionChanged(option, it) },
                    )
                }
            }
    }
}
