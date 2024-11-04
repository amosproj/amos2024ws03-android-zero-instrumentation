package de.amosproj3.ziofa.ui.visualization

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import de.amosproj3.ziofa.client.ClientFactory
import org.koin.compose.koinInject

/** Screen for visualizing data. */
@Composable
fun VisualizationScreen(
    modifier: Modifier = Modifier,
    clientFactory: ClientFactory = koinInject(),
) {
    Box(modifier = modifier) {
        Text("This is the visualization screen")
        Counter(clientFactory = clientFactory)
    }
}
