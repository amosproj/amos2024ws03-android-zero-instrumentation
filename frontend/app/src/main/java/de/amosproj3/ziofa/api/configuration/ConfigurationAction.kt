package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

sealed class ConfigurationAction {
    data object Synchronize : ConfigurationAction()

    data class ChangeFeature(
        val backendFeature: BackendFeatureOptions,
        val enable: Boolean,
        val pids: Set<UInt>
    ) : ConfigurationAction()

    data object Reset : ConfigurationAction()
}