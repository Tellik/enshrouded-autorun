# Gestor de DLL

Aquí puedes ver qué DLL de mods ha encontrado ShroudForge y si se han cargado. Los mods conocidos tienen perfiles preparados. Para otras DLL, selecciónalas y confírmalas una vez en el Gestor de DLL.

## ¿Qué hago con el mod descargado?

- **No incluye DLL:** Instala el mod como siempre. Los mods creados para ShroudForge o EML no necesitan una DLL nativa adicional.
- **Incluye una DLL:** Coloca la DLL del mod en su carpeta, como se muestra abajo. Después aparecerá en el Gestor de DLL. Los perfiles conocidos ya tienen preparada su selección. Si no hay perfil, selecciona la DLL y confírmala allí.
- **Incluye otros archivos:** Conserva la carpeta completa del mod. No borres archivos como `mod.json`, `native-plugin.ini` o los archivos de configuración.

## Si el mod contiene varias DLL

Muchas carpetas de mods contienen DLL que solo sirven como archivos auxiliares. Selecciona únicamente las DLL que el perfil o las instrucciones del autor indiquen que deben cargarse. Si hay que cargar varias DLL del mod, establece el orden en el Gestor de DLL.

## Instalar un mod con DLL

Si el instalador de ShroudForge reconoce el paquete descargado, instálalo allí. Para instalarlo manualmente, crea una carpeta propia para cada mod directamente dentro de `mods/` y coloca allí la DLL del mod. Si la DLL está junto a la carpeta del mod en el archivo, colócala junto al mod. Conserva el nombre original y las subcarpetas incluidas en el paquete.

```text
mods/
  mi-mod/
    mod.json        (si está incluido)
    dinput8.dll     (o la DLL del mod)
    config.ini      (si está incluido)
```

Una DLL desconocida no se carga solo por estar en la carpeta. Selecciónala en el Gestor de DLL, comprueba el modo de carga sugerido y confirma la selección.

## ¿Por qué la DLL del mod también puede llamarse dinput8.dll?

El archivo de inicio de ShroudForge es `dinput8.dll` en la carpeta del juego. El archivo del mod está separado en `mods/<carpeta-del-mod>/dinput8.dll`. ShroudForge lo carga desde esa ruta completa. Con un perfil o una selección adecuada en el Gestor de DLL, ShroudForge reenvía las llamadas DirectInput al mod. Ambos archivos pueden conservar el mismo nombre. No cambies el nombre de la DLL del mod ni la pongas en la carpeta del juego. No sustituyas allí el archivo de inicio de ShroudForge.

## Entender los perfiles

Un perfil es una configuración preparada para un mod conocido. Indica a ShroudForge qué archivos cargar y cómo hacerlo. ShroudForge lo relaciona con el identificador del mod en `mod.json` o, si no existe, con el nombre de la carpeta. También puedes elegir un perfil en el Gestor de DLL. Los perfiles no modifican los archivos del mod. Una selección propia guardada sustituye la lista de archivos del perfil.

## Cadena DirectInput existente

Un `dinput8-chain.ini` existente en la carpeta del juego utiliza el formato establecido:

```ini
[Chain]
mods\mi-mod\dinput8.dll
mods\otro-mod\native.dll
```

Las entradas deben apuntar a archivos dentro de paquetes de mods directos como `mods/<carpeta-del-mod>/`. Se rechazan las DLL del directorio raíz del juego y las rutas fuera de esos paquetes. Su orden tiene prioridad sobre las declaraciones calculadas automáticamente. Una misma ruta resuelta se carga una sola vez. La primera implementación DirectInput válida recibe las llamadas reenviadas. Se omiten los archivos de inicio propios y el reenvío recursivo recurre a la biblioteca original de Windows.

## Interruptores, proceso de destino y reinicio

El interruptor principal controla toda la carga nativa realizada por ShroudForge, incluidos INI, perfiles, cadenas y registros Lua. Una selección desactivada impide la carga automática de esa carpeta. Cliente, servidor o ambos determina el proceso de destino. Las exclusiones por nombre se aplican justo antes de cargar. No identifican las funciones de un mod a partir de su nombre.

Las DLL cargadas permanecen en memoria hasta que termina el proceso. Desactivarlas, cambiar su orden o elegir otro perfil requiere reiniciar el juego. Las declaraciones duplicadas de un archivo no crean otra instancia.

## Aparece un error de carga

Comprueba el nombre del mod, la ruta y la causa. El error 126 de Windows puede indicar que falta el archivo principal o alguna biblioteca necesaria. Si falta la función exportada de DirectInput, comprueba el archivo seleccionado y su modo de carga. «Cargado» confirma la carga, no todas las funciones del mod.

Un formato de plugin nativo desconocido, una capa Vulkan, un cargador completo adicional o un programa externo necesita su integración prevista. Cargar una biblioteca no sustituye esa interfaz.

## Instalar paquetes

El instalador también encuentra un único paquete de mod en carpetas anidadas del archivo. Los perfiles conocidos importan archivos auxiliares indicados expresamente. Una nueva instalación no instala automáticamente en conjunto un archivo con varios paquetes de mods. Descomprímelo e instala una carpeta de mod cada vez. El funcionamiento normal no abre archivos comprimidos ni inicia programas externos.
