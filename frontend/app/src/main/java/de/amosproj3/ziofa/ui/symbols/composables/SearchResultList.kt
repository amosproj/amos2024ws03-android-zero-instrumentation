// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.symbols.composables

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
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry

@Composable
fun SearchResultList(
    symbols: Map<SymbolsEntry, Boolean>,
    onOptionChanged: (SymbolsEntry, Boolean) -> Unit,
) {

    LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
        item { Spacer(Modifier.height(15.dp)) }

        items(symbols.entries.toList().sortedBy { it.key.name }) { option ->
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.key.name, modifier = Modifier.weight(10f))
                Checkbox(
                    modifier = Modifier.weight(1f),
                    checked = option.value,
                    onCheckedChange = { onOptionChanged(option.key, it) },
                )
            }
        }
    }
}
