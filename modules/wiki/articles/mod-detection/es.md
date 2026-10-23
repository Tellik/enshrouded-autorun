# Detección de mods

Consulta qué carpetas y archivos reconoce ShroudForge. La detección no carga DLL ni modifica los archivos de los mods.

## ¿Qué se detecta?

Los mods instalados son carpetas descomprimidas en `mods/<carpeta-del-mod>/` o en el directorio configurado. ShroudForge lee `mod.json`, el archivo opcional `extended.mod.json`, `src/mod.lua`, `native-plugin.ini` y las DLL existentes. Un mod que solo usa Lua no necesita su propia DLL. Un mod nativo puede aparecer en el Gestor de DLL sin manifiesto ni archivo Lua.

Esta vista indica si se han reconocido la carpeta y sus datos. La carga nativa real se muestra en el Gestor de DLL. El estado general del mod aparece en su propia página.

## Campos desconocidos del manifiesto

Con el módulo de detección y la lectura tolerante activados, `mod.json` y `extended.mod.json` pueden incluir campos que ShroudForge desconoce. Esos campos por sí solos no provocan el rechazo del paquete. Los campos conocidos deben seguir teniendo valores válidos. El JSON incorrecto, los datos obligatorios ausentes y los tipos erróneos siguen siendo errores.

Desactivar esta compatibilidad restaura la validación estricta. No elimina ni desactiva automáticamente todos los mods instalados.

## No se reconoce un mod

Comprueba primero la carpeta real del mod. Una descarga puede contener otra carpeta envolvente o un directorio `mods`. El instalador localiza un único paquete de mod. Una nueva instalación no instala automáticamente en conjunto un archivo con varios paquetes de mods. Descomprímelo e instala una carpeta de mod cada vez. Durante el funcionamiento normal se leen carpetas descomprimidas, no archivos comprimidos.

Corrige la ubicación JSON indicada o vuelve a instalar el paquete completo. Un perfil de compatibilidad puede adaptar diferencias conocidas. Reconocer el mod no confirma que sus hooks nativos o sus funciones de API funcionen.
