// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes.composables

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.google.accompanist.drawablepainter.rememberDrawablePainter
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.ui.shared.toReadableString

@Composable
fun IconAndName(option: RunningComponent.Application, modifier: Modifier = Modifier) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically) {
        val painter = rememberDrawablePainter(option.packageInfo.icon)
        Image(painter = painter, contentDescription = "", modifier = Modifier.size(50.dp, 50.dp))
        Spacer(modifier = Modifier.width(20.dp))
        Text(text = option.packageInfo.displayName)
    }
}

@Composable
fun IconAndName(option: RunningComponent.StandaloneProcess, modifier: Modifier = Modifier) {
    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically) {
        Image(
            imageVector = Icons.Filled.Info,
            contentDescription = "",
            modifier = Modifier.size(50.dp, 50.dp),
        )
        Spacer(modifier = Modifier.width(20.dp))
        Text(text = option.process.cmd.toReadableString(), modifier = modifier)
    }
}
