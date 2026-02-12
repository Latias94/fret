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
        String diag = getIntent().getStringExtra("FRET_DIAG");
        boolean diagEnabled = diag != null && !diag.trim().isEmpty();
        if (diag != null && !diag.trim().isEmpty()) {
            try {
                Os.setenv("FRET_DIAG", diag.trim(), true);
            } catch (ErrnoException e) {
                Log.w("fret", "failed to set FRET_DIAG", e);
            }
        }

        String diagDir = getIntent().getStringExtra("FRET_DIAG_DIR");
        if (diagEnabled && (diagDir == null || diagDir.trim().isEmpty())) {
            diagDir = getFilesDir().getAbsolutePath() + "/fret-diag";
        }
        if (diagDir != null && !diagDir.trim().isEmpty()) {
            try {
                Os.setenv("FRET_DIAG_DIR", diagDir.trim(), true);
            } catch (ErrnoException e) {
                Log.w("fret", "failed to set FRET_DIAG_DIR", e);
            }
        }

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
