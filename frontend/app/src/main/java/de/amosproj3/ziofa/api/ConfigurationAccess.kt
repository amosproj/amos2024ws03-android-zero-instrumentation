package de.amosproj3.ziofa.api

import kotlinx.coroutines.flow.StateFlow
import uniffi.shared.Configuration

interface ConfigurationAccess {
    val configuration: StateFlow<ConfigurationUpdate>
    fun submitConfiguration(configuration: Configuration)
}