// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.symbols.composables

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun SearchBar(value: String, onValueChanged: (String) -> Unit, onStartSearch: (String) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            modifier = Modifier.weight(3f),
            value = value,
            onValueChange = { onValueChanged(it) },
            placeholder = { Text("Enter symbol name") },
        )
        Button(
            shape = RoundedCornerShape(10),
            modifier = Modifier.weight(1f).padding(start = 10.dp),
            onClick = { onStartSearch(value) },
            content = {
                Icon(imageVector = Icons.Filled.Search, contentDescription = "")
                Text("Search")
            },
        )
    }
}
