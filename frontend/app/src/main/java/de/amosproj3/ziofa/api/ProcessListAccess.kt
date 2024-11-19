package de.amosproj3.ziofa.api

import kotlinx.coroutines.flow.StateFlow
import uniffi.shared.Process

interface ProcessListAccess {
    val processesList: StateFlow<List<Process>>
}