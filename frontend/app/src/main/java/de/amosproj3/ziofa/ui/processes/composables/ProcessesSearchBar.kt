// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes.composables

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp

@Composable
fun ProcessesSearchBar(
    value: String,
    onValueChanged: (String) -> Unit,
    onStartSearch: (String) -> Unit,
) {

    val keyboardController = LocalSoftwareKeyboardController.current

    Row(
        modifier = Modifier.background(MaterialTheme.colorScheme.primary),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = value,
            onValueChange = {
                if (it.lastOrNull() == '\n') {
                    keyboardController?.hide()
                    onStartSearch(value)
                } else {
                    onValueChanged(it)
                }
            },
            modifier = Modifier.weight(8f).background(Color.White),
            keyboardOptions = KeyboardOptions.Default.copy(imeAction = ImeAction.Done),
            keyboardActions =
                KeyboardActions(
                    onDone = {
                        keyboardController?.hide()
                        onStartSearch(value)
                    }
                ),
            placeholder = { Text("Search for processes and apps ... ") },
        )
        Icon(
            imageVector = Icons.Filled.Search,
            contentDescription = "",
            modifier =
                Modifier.weight(1f).padding(10.dp).size(20.dp).clickable { onStartSearch(value) },
            tint = Color.White,
        )
    }
    HorizontalDivider(Modifier.height(15.dp).background(MaterialTheme.colorScheme.primary))
}
