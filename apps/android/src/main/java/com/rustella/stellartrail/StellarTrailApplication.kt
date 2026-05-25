package com.rustella.stellartrail

import android.app.Application
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.di.DefaultAppContainer

class StellarTrailApplication : Application() {
    lateinit var container: AppContainer
        private set

    override fun onCreate() {
        super.onCreate()
        container = DefaultAppContainer(this)
    }
}
