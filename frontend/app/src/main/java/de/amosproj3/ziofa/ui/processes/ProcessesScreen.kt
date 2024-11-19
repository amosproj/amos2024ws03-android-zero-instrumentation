package de.amosproj3.ziofa.ui.processes

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.koin.androidx.compose.koinViewModel
import uniffi.shared.Cmd

@Composable
fun ProcessesScreen(modifier: Modifier, viewModel: ProcessesViewModel = koinViewModel()) {
    Box(modifier = modifier.fillMaxSize()) {

        Column {
            Row(modifier = Modifier.padding(horizontal = 20.dp, vertical = 10.dp)) {
                Text(text = "CMD", modifier = Modifier.weight(1f))
                Text(text = "State", modifier = Modifier.weight(1f))
                Text(text = "PID", modifier = Modifier.weight(1f))
                Text(text = "Parent PID", modifier = Modifier.weight(1f))
            }

            val options by remember { viewModel.processesList }.collectAsState()
            LazyColumn(
                modifier = Modifier
                    .padding(horizontal = 20.dp)
                    .fillMaxSize()
            ) {
                items(options) { option ->
                    Row(
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.SpaceEvenly,
                        verticalAlignment = Alignment.Top,
                    ) {
                        Text(text = option.cmd.toReadableString(), modifier = Modifier.weight(1f))
                        Text(text = option.state, modifier = Modifier.weight(1f))
                        Text(text = option.pid.toString(), modifier = Modifier.weight(1f))
                        Text(text = option.ppid.toString(), modifier = Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

fun Cmd?.toReadableString(): String {
    this?.let {
        return when (this) {
            is Cmd.Comm -> this.v1
            is Cmd.Cmdline -> this.v1.args.joinToString(" ")
        }
    } ?: return "null"

}