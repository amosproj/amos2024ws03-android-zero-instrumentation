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
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.data.EbpfProgramOptions

@Composable
fun EbpfOptions(
    options: EbpfProgramOptions,
    onVfsWriteChanged: (Boolean) -> Unit,
    onSendMessageChanged: (Boolean) -> Unit,
) {
    LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
        item { Spacer(Modifier.height(15.dp)) }

        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Vfs Write Analysis")
                Checkbox(
                    checked = options.vfsWriteOption.enabled,
                    onCheckedChange = onVfsWriteChanged,
                )
            }
        }

        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Send Message Analysis")
                Checkbox(
                    checked = options.sendMessageOption.enabled,
                    onCheckedChange = onSendMessageChanged,
                )
            }
        }
    }
}
