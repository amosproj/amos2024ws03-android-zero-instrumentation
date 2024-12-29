package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig

sealed class ConfigurationAction {
    data object Synchronize : ConfigurationAction()
    data class Change(
        val enable: Boolean,
        val vfsWriteFeature: VfsWriteConfig? = null,
        val sendMessageFeature: SysSendmsgConfig? = null,
        val uprobesFeature: List<UprobeConfig>? = listOf(),
        val jniReferencesFeature: JniReferencesConfig? = null
    ) : ConfigurationAction()
    data object Reset: ConfigurationAction()
}