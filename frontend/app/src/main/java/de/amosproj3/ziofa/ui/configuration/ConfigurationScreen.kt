package de.amosproj3.ziofa.ui.configuration

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

/** Screen for configuring eBPF programs */
@Composable
fun ConfigurationScreen(modifier: Modifier = Modifier) {
    Box(modifier = modifier) { Text("This is the configuration screen") }
}
