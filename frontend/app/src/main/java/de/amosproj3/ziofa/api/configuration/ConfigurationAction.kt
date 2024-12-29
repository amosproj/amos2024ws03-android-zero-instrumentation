package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

/** Possible interaction with the configuration */
sealed class ConfigurationAction {
    /** Overwrite the backend configuration with the in-memory configuration */
    data object Synchronize : ConfigurationAction()

    /** [enable] or disable a [backendFeature] for given [pids]. */
    data class ChangeFeature(
        val backendFeature: BackendFeatureOptions,
        val enable: Boolean,
        val pids: Set<UInt>
    ) : ConfigurationAction()

    /** Reset the backend configuration to an empty configuration. */
    data object Reset : ConfigurationAction()
}