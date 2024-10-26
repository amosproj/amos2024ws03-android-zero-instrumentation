package de.amosproj3.ziofa

import android.app.Application
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.startKoin
import org.koin.dsl.module

class ZIOFAApplication : Application() {

    val appModule = module {
        // TODO add declarations here
    }

    override fun onCreate() {
        super.onCreate()

        startKoin {
            androidLogger()
            androidContext(this@ZIOFAApplication)
            modules(
                appModule
            )
        }
    }
}