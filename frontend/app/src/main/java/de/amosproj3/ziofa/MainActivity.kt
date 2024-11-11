// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.ui.theme.ZIOFATheme
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import org.koin.android.ext.android.inject
import org.koin.androidx.compose.KoinAndroidContext

class MainActivity : ComponentActivity() {

    private val clientFactory: ClientFactory by inject()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            ZIOFATheme {
                KoinAndroidContext {
                    Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                        Counter(clientFactory, modifier = Modifier.padding(innerPadding))
                    }
                }
            }
        }
    }
}

@Composable
fun Counter(clientFactory: ClientFactory, modifier: Modifier = Modifier) {
    var client by remember { mutableStateOf<Client?>(null) }
    var waiting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    var maybeCount by remember { mutableStateOf(0u) }

    LaunchedEffect(client) {
        client?.loadProgram("example")
        client?.serverCount?.collect { maybeCount = it }
    }

    Box(contentAlignment = Alignment.Center, modifier = modifier.fillMaxSize()) {
        if (client != null) {
            Text("$maybeCount")
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(
                    onClick = {
                        waiting = true
                        error = null
                        scope.launch {
                            try {
                                client = clientFactory.connect(scope, "http://[::1]:50051")
                            } catch (e: Exception) {
                                error = e.message
                            }
                            waiting = false
                        }
                    },
                    enabled = !waiting,
                ) {
                    Text("Connect")
                }
                error?.let { Text(it, style = LocalTextStyle.current.copy(color = Color.Red)) }
            }
        }
    }
}
