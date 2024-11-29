// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.app.Application
import android.content.Context
import android.content.pm.PackageManager
import de.amosproj3.ziofa.api.BackendConfigurationAccess
import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.api.LocalConfigurationAccess
import de.amosproj3.ziofa.api.RunningComponentsAccess
import de.amosproj3.ziofa.bl.ConfigurationManager
import de.amosproj3.ziofa.bl.DataStreamManager
import de.amosproj3.ziofa.bl.PackageInformationProvider
import de.amosproj3.ziofa.bl.RunningComponentsProvider
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.RustClientFactory
import de.amosproj3.ziofa.ui.configuration.ConfigurationViewModel
import de.amosproj3.ziofa.ui.processes.ProcessesViewModel
import de.amosproj3.ziofa.ui.visualization.VisualizationViewModel
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
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
        single<DataStreamProvider> { DataStreamManager(get()) }

        single { ConfigurationManager(clientFactory = get()) } binds
            arrayOf(BackendConfigurationAccess::class, LocalConfigurationAccess::class)
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
                dataStreamProvider = get(),
                runningComponentsAccess = get(),
            )
        }
    }
}
