// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes.composables

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ProcessesHeader() {
    Row(modifier = Modifier.padding(horizontal = 20.dp, vertical = 10.dp)) {
        Text(text = "Name", modifier = Modifier.weight(1f))
        Text(text = "PID(s)", modifier = Modifier.weight(1f))
        Text(text = "Parent PID", modifier = Modifier.weight(1f))
        Text(text = "", modifier = Modifier.weight(1f))
    }
    HorizontalDivider(Modifier.height(3.dp).background(MaterialTheme.colorScheme.primary))
}
