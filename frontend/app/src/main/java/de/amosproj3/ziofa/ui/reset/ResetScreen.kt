// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.reset

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import org.koin.androidx.compose.koinViewModel

const val RESET_WARNING =
    """
    Do you really want to reset the configuration?
    
    
    This will set all features to disabled for all processes!
"""

@Composable
fun ResetScreen(
    modifier: Modifier,
    viewModel: ResetViewModel = koinViewModel(),
    afterResetConfirmed: () -> Unit,
) {

    Box(modifier = modifier.fillMaxSize()) {
        Text(
            RESET_WARNING,
            modifier = Modifier.align(Alignment.Center),
            fontSize = TextUnit(40f, TextUnitType.Sp),
            fontWeight = FontWeight.Bold,
        )
        Button(
            modifier = Modifier.align(Alignment.BottomCenter).padding(20.dp).fillMaxWidth(),
            onClick = {
                viewModel.reset()
                afterResetConfirmed()
            },
        ) {
            Text("Reset configuration", fontSize = TextUnit(40f, TextUnitType.Sp))
        }
    }
}
