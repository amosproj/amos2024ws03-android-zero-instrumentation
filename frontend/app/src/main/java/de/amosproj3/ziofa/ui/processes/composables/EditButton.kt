// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes.composables

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier

@Composable
fun EditButton(modifier: Modifier = Modifier, onClick: () -> Unit = {}) {
    Box(modifier = modifier.clickable { onClick() }) {
        Image(
            imageVector = Icons.Filled.Edit,
            contentDescription = "",
            modifier = Modifier.align(Alignment.CenterEnd),
        )
    }
}
