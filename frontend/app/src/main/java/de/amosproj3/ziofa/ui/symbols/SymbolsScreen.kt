package de.amosproj3.ziofa.ui.symbols

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import de.amosproj3.ziofa.ui.symbols.data.SymbolsScreenState
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview(device = Devices.AUTOMOTIVE_1024p)
fun SymbolsScreen(modifier: Modifier = Modifier, pid: UInt = 123u) { // TODO pass pid to screen
    val viewModel: SymbolsViewModel = koinViewModel(parameters = { parametersOf(pid) })
    // TODO render two lists, if both have selected an item, show confirm button
    // TODO upon clicking, add to local configuration to show on configuration screen
    // TODO from there, we can submit it :)

    //TODO possibly searchable symbols if time

    // TODO configurationscreen should render uprobes as odex - method name or smth

    var searchQuery by remember { mutableStateOf("") }

    Box(modifier = modifier) {
        Row(modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(
                modifier = Modifier.weight(2f),
                value = searchQuery,
                onValueChange = {
                    searchQuery = it
                }, placeholder = {
                    Text("Enter symbol name")
                }
            )
            Button(modifier = Modifier.weight(1f),
                onClick = {
                    //TODO
                },
                content = {
                    Text("Search")
                })
        }
    }

    val searchResultsState =
        viewModel.searchResult.collectAsState(SymbolsScreenState.WaitingForSearch)

    Box(Modifier.fillMaxSize()) {
        when (val searchResults = searchResultsState.value) {
            is SymbolsScreenState.SymbolsLoading -> CircularProgressIndicator()
            is SymbolsScreenState.SearchResultReady -> SearchResultList(
                searchResults.symbols,
                onOptionChanged = { symbol, active -> viewModel.symbolEntryChanged(symbol, active) }
            )

            is SymbolsScreenState.WaitingForSearch -> Spacer(Modifier.fillMaxSize())
            is SymbolsScreenState.Error -> ErrorScreen(error = searchResults.errorMessage)
        }
    }


}

@Composable
fun SearchResultList(
    symbols: List<SymbolsEntry>,
    onOptionChanged: (SymbolsEntry, Boolean) -> Unit
) {

    LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
        item { Spacer(Modifier.height(15.dp)) }

        items(symbols) { option ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.name)
                Checkbox(
                    checked = option.active,
                    onCheckedChange = { onOptionChanged(option, it) })
            }
        }
    }
}