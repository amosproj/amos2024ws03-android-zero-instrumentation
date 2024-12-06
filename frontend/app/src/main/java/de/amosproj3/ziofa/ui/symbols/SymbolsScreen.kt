package de.amosproj3.ziofa.ui.symbols

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import de.amosproj3.ziofa.ui.configuration.composables.SubmitFab
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import de.amosproj3.ziofa.ui.symbols.data.SymbolsScreenState
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
fun SymbolsScreen(
    modifier: Modifier = Modifier,
    onSymbolsSubmitted: () -> Unit,
    pids: List<UInt> = listOf(123u)
) { // TODO pass pid to screen
    val viewModel: SymbolsViewModel = koinViewModel(parameters = { parametersOf(pids) })

    var searchQuery by remember { mutableStateOf("") }
    val screenState = viewModel.screenState.collectAsState(SymbolsScreenState.WaitingForSearch)

    Box(modifier = modifier) {
        Column(modifier = Modifier.fillMaxWidth()) {
            SearchBar(
                value = searchQuery,
                onValueChanged = { searchQuery = it },
                onStartSearch = { viewModel.startSearch(it) }
            )


            Box(Modifier.fillMaxSize()) {
                when (val state = screenState.value) {
                    is SymbolsScreenState.SymbolsLoading -> CircularProgressIndicator(
                        Modifier.align(
                            Alignment.Center
                        )
                    )

                    is SymbolsScreenState.SearchResultReady -> SearchResultList(
                        state.symbols,
                        onOptionChanged = { symbol, active ->
                            viewModel.symbolEntryChanged(
                                symbol,
                                active
                            )
                        }
                    )


                    is SymbolsScreenState.WaitingForSearch -> Spacer(Modifier.fillMaxSize())
                    is SymbolsScreenState.Error -> ErrorScreen(error = state.errorMessage)
                }
            }
        }
        SubmitFab(
            modifier = Modifier.align(Alignment.BottomEnd), onClick = {
                viewModel.submit()
                onSymbolsSubmitted()
            })
    }


}

@Composable
fun SearchBar(value: String, onValueChanged: (String) -> Unit, onStartSearch: (String) -> Unit) {
    Row(modifier = Modifier.fillMaxWidth()) {
        OutlinedTextField(
            modifier = Modifier.weight(2f),
            value = value,
            onValueChange = {
                onValueChanged(it)
            }, placeholder = {
                Text("Enter symbol name")
            }
        )
        Button(modifier = Modifier.weight(1f),
            onClick = {
                onStartSearch(value)
            },
            content = {
                Text("Search")
            })
    }
}

@Composable
fun SearchResultList(
    symbols: Map<SymbolsEntry, Boolean>,
    onOptionChanged: (SymbolsEntry, Boolean) -> Unit
) {

    LazyColumn(
        modifier = Modifier
            .padding(horizontal = 20.dp)
            .fillMaxSize()
    ) {
        item { Spacer(Modifier.height(15.dp)) }

        items(symbols.entries.toList().sortedBy { it.key.name }) { option ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.key.name)
                Checkbox(
                    checked = option.value,
                    onCheckedChange = { onOptionChanged(option.key, it) })
            }
        }
    }
}