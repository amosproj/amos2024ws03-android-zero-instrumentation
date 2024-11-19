package de.amosproj3.ziofa.api

import uniffi.shared.Configuration

sealed class ConfigurationUpdate {
    data class OK(val configuration: Configuration) : ConfigurationUpdate()
    data class NOK(val error: Throwable) : ConfigurationUpdate()
    data object UNKNOWN : ConfigurationUpdate()
}

