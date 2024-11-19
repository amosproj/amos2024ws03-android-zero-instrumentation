package de.amosproj3.ziofa.ui.configuration.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.data.EBpfProgramOption

@Composable
fun EbpfOptions(
    options: List<EBpfProgramOption>,
    onCheckedChanged: (EBpfProgramOption, Boolean) -> Unit
) {
    LazyColumn(
        modifier = Modifier
            .padding(horizontal = 20.dp)
            .fillMaxSize()
    ) {
        item { Spacer(Modifier.height(15.dp)) }

        items(options) { option ->
            Row(
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(option.name)
                Checkbox(
                    checked = option.active,
                    onCheckedChange = { onCheckedChanged(option, it) }
                )
            }
        }
    }
}