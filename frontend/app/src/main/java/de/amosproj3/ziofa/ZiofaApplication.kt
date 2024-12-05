// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.app.Application
import android.content.Context
import android.content.pm.PackageManager
import de.amosproj3.ziofa.api.configuration.BackendConfigurationAccess
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.configuration.LocalConfigurationAccess
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.bl.configuration.ConfigurationManager
import de.amosproj3.ziofa.bl.events.DataStreamManager
import de.amosproj3.ziofa.bl.processes.PackageInformationProvider
import de.amosproj3.ziofa.bl.processes.RunningComponentsProvider
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.RustClientFactory
import de.amosproj3.ziofa.ui.configuration.ConfigurationViewModel
import de.amosproj3.ziofa.ui.processes.ProcessesViewModel
import de.amosproj3.ziofa.ui.visualization.VisualizationViewModel
import kotlinx.coroutines.CoroutineScope
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.core.parameter.parametersOf
import org.koin.dsl.binds
import org.koin.dsl.module
import timber.log.Timber

class ZiofaApplication : Application() {

    val appModule: Module = module {
        createExternalDependencies()
        createBLModules()
        createViewModelFactories()
    }

    override fun onCreate() {
        super.onCreate()
        Timber.plant(Timber.DebugTree()) // start Timber logging

        startKoin {
            androidLogger()
            androidContext(this@ZiofaApplication)
            modules(appModule)
        }
    }

    private fun Module.createExternalDependencies() {
        single<ClientFactory> { RustClientFactory("http://[::1]:50051") }
        single<PackageManager> { get<Context>().packageManager }
    }

    private fun Module.createBLModules() {
        single { PackageInformationProvider(get()) }
        single<RunningComponentsAccess> {
            RunningComponentsProvider(clientFactory = get(), packageInformationProvider = get())
        }
        single { ConfigurationManager(clientFactory = get()) } binds
            arrayOf(BackendConfigurationAccess::class, LocalConfigurationAccess::class)
        factory<DataStreamProvider> { (scope: CoroutineScope) -> DataStreamManager(get(), scope) }
    }

    private fun Module.createViewModelFactories() {
        viewModel { (pids: List<UInt>) ->
            ConfigurationViewModel(
                backendConfigurationAccess = get(),
                localConfigurationAccess = get(),
                pids = pids,
            )
        }
        viewModel { ProcessesViewModel(runningComponentsProvider = get()) }
        viewModel {
            VisualizationViewModel(
                backendConfigurationAccess = get(),
                dataStreamProviderFactory = { get { parametersOf(it) } },
                runningComponentsAccess = get(),
            )
        }
    }
}
