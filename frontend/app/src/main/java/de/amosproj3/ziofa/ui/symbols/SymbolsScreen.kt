package de.amosproj3.ziofa.ui.symbols

import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview(device = Devices.AUTOMOTIVE_1024p)
fun SymbolsScreen(pid: UInt = 123u) { // TODO pass pid to screen
    val viewModel: SymbolsViewModel = koinViewModel(parameters = { parametersOf(pid) })
    // TODO render two lists, if both have selected an item, show confirm button
    // TODO upon clicking, add to local configuration to show on configuration screen
    // TODO from there, we can submit it :)

    //TODO possibly searchable symbols if time

    // TODO configurationscreen should render uprobes as odex - method name or smth
}