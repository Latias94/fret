package dev.fret.ui_gallery;

import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;

import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("fret_ui_gallery_mobile");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        String backend = getIntent().getStringExtra("FRET_WGPU_BACKEND");
        if (backend != null && !backend.trim().isEmpty()) {
            // Must run before the Rust side initializes wgpu.
            try {
                Os.setenv("FRET_WGPU_BACKEND", backend.trim(), true);
            } catch (ErrnoException e) {
                Log.w("fret", "failed to set FRET_WGPU_BACKEND", e);
            }
        }
        super.onCreate(savedInstanceState);
    }
}
