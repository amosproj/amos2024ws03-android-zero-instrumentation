package de.amosproj3.ziofa

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.view.Gravity
import android.view.WindowManager
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private val REQUEST_CODE_OVERLAY_PERMISSION = 1234

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.layout)

        showOverlay()  // Show overlay if permission is granted
    }

    // Handle the result of the permission request
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
    }

    // Function to show the overlay
    private fun showOverlay() {
        val windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        val overlayText = TextView(this).apply {
            text = "Hello World!"
            textSize = 24f
            setTextColor(resources.getColor(android.R.color.white))
        }

        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,  // Overlay window type
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE,  // Ensures it's non-interactive
            android.graphics.PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START  // Position the overlay at the top-left
            x = 0
            y = 0
        }

        windowManager.addView(overlayText, params)  // Add the overlay to the screen
    }
}