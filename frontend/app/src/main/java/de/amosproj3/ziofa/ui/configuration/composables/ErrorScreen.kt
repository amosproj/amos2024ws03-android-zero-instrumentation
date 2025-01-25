// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.composables

import android.os.Process
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import kotlin.system.exitProcess

const val TITLE_TEXT_SIZE = 25f

@Preview(device = Devices.AUTOMOTIVE_1024p)
@Composable
fun ErrorScreen(
    error: String = "No error message available",
    title: String = "Error while communicating with backend",
    modifier: Modifier = Modifier,
) {
    Box(modifier = modifier.fillMaxSize()) {
        Column(
            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = title,
                color = Color.Red,
                fontSize = TextUnit(TITLE_TEXT_SIZE, TextUnitType.Sp),
            )
            Text(text = error)
        }
        Button(
            onClick = {
                Process.killProcess(Process.myPid())
                exitProcess(1)
            },
            modifier =
                Modifier.fillMaxWidth().align(Alignment.BottomCenter).padding(horizontal = 20.dp),
        ) {
            Text("Exit application")
        }
    }
}
