// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.init

import android.graphics.drawable.AdaptiveIconDrawable
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import androidx.core.content.res.ResourcesCompat
import androidx.core.graphics.drawable.toBitmap
import de.amosproj3.ziofa.R
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.init.data.InitState
import org.koin.androidx.compose.koinViewModel

/**
 * Screen displayed on startup. The loading screen could be displayed while indexing symbols in the
 * future.
 */
@Composable
fun InitScreen(
    onInitFinished: () -> Unit,
    modifier: Modifier,
    viewModel: InitViewModel = koinViewModel(),
) {
    val initState: InitState by remember { viewModel.initState }.collectAsState()
    Box(modifier.fillMaxSize()) {
        when (val state = initState) {
            is InitState.Initializing -> LoadingScreen(Modifier.align(Alignment.Center))
            is InitState.Error ->
                Column(Modifier.align(Alignment.Center)) {
                    ErrorScreen(
                        error = state.error,
                        title =
                            "Are you sure the backend is started? An error occured while connecting to the backend",
                    )
                }
            is InitState.Initialized -> onInitFinished()
        }
    }
}

@Composable
private fun LoadingScreen(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = "Welcome to",
            fontWeight = FontWeight.Bold,
            fontSize = TextUnit(30f, TextUnitType.Sp),
        )

        Row(
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val ourLogo =
                ResourcesCompat.getDrawable(
                    LocalContext.current.resources,
                    R.mipmap.ic_launcher,
                    LocalContext.current.theme,
                ) as AdaptiveIconDrawable
            val bitmapPainter = BitmapPainter(ourLogo.toBitmap().asImageBitmap())
            Image(
                painter = bitmapPainter,
                contentDescription = "",
                modifier = Modifier.size(100.dp),
            )
            Text(
                text = "Zero Instrumentation Observability for Android",
                fontWeight = FontWeight.Bold,
                fontSize = TextUnit(50f, TextUnitType.Sp),
            )
            Image(
                painter = bitmapPainter,
                contentDescription = "",
                modifier = Modifier.size(100.dp),
            )
        }
        CircularProgressIndicator(Modifier.padding(70.dp).size(100.dp))
        Text(text = "Connecting to the backend ...", fontStyle = FontStyle.Italic)
    }
}
