/**
 * X - Cliente no oficial de X (Twitter) para macOS
 * Copyright © 2024 686f6c61
 *
 * @author 686f6c61 (https://github.com/686f6c61)
 * @repository https://github.com/686f6c61/Xcom-mac-silicon
 * @description Main frontend script - Maneja el diálogo de ayuda.
 * Las demás funcionalidades (navegación, recarga, etc.) se gestionan
 * mediante el menú nativo de macOS.
 */

const { invoke } = window.__TAURI__.core;

/**
 * Inyecta el detector de login en el iframe de X
 * @description
 * El detector monitorea cuando el usuario hace login en X.com
 * y guarda automáticamente las credenciales en el sistema multicuenta.
 */
function injectLoginDetector() {
  const iframe = document.getElementById('twitter-frame');
  if (!iframe) {
    console.warn('[Main] Twitter iframe not found, skipping login detector injection');
    return;
  }

  iframe.addEventListener('load', () => {
    try {
      // Intentar inyectar el script en el iframe
      const script = iframe.contentDocument.createElement('script');
      script.src = '/login-detector.js';
      script.type = 'module';
      iframe.contentDocument.head.appendChild(script);
      console.log('[Main] Login detector injected successfully');
    } catch (e) {
      // CORS puede prevenir el acceso al iframe
      console.warn('[Main] Failed to inject login detector (CORS?):', e);
    }
  });
}

/**
 * Escucha mensajes del iframe de X
 * @listens message - window
 * @description
 * Maneja mensajes enviados desde el login-detector.js
 * para reconstruir el menú cuando se agrega una cuenta.
 */
window.addEventListener('message', async (event) => {
  // Verificar origen
  if (event.origin !== 'https://x.com' && event.origin !== 'https://twitter.com') {
    return;
  }

  // Manejar evento de cuenta agregada
  if (event.data.type === 'account-added') {
    try {
      console.log('[Main] Account added:', event.data.username);
      // Reconstruir el menú para mostrar la nueva cuenta
      await invoke('rebuild_accounts_menu');
      console.log('[Main] Menu rebuilt successfully');
    } catch (e) {
      console.error('[Main] Failed to rebuild menu:', e);
    }
  }
});


/**
 * Inicializa event listeners para el diálogo de ayuda.
 *
 * @listens DOMContentLoaded
 * @description
 * Configura listeners para:
 * - Botón de cierre de ayuda (cierra modal)
 * - Click fuera del modal (cierra modal)
 * - Tecla Escape (cierra modal si está abierto)
 */
document.addEventListener('DOMContentLoaded', async () => {
  try {
    const helpDialog = document.getElementById('helpDialog');
    const closeHelp = document.getElementById('closeHelp');

    if (!helpDialog || !closeHelp) {
      console.error('Error: No se encontraron elementos del diálogo');
      return;
    }

    /**
     * Cierra el diálogo modal de ayuda.
     * @listens click - closeHelp button
     */
    closeHelp.addEventListener('click', () => {
      try {
        helpDialog.close();
      } catch (error) {
        console.error('Error cerrando diálogo de ayuda:', error);
      }
    });

    /**
     * Cierra el modal si se hace click fuera de él (en el backdrop).
     * @listens click - helpDialog backdrop
     */
    helpDialog.addEventListener('click', (e) => {
      if (e.target === helpDialog) {
        try {
          helpDialog.close();
        } catch (error) {
          console.error('Error cerrando diálogo:', error);
        }
      }
    });

    /**
     * Cierra el modal con la tecla Escape.
     * @listens keydown - window
     */
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && helpDialog.open) {
        try {
          helpDialog.close();
        } catch (error) {
          console.error('Error cerrando diálogo con Escape:', error);
        }
      }
    });

    // Inyectar detector de login en el iframe de X
    injectLoginDetector();

  } catch (error) {
    console.error('Error fatal en inicialización:', error);
  }
});
