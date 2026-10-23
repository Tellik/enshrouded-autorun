# Détection des mods

Consultez les dossiers et fichiers reconnus par ShroudForge. La détection ne charge aucune DLL et ne modifie pas les fichiers des mods.

## Que détecte ShroudForge ?

Les mods installés sont des dossiers décompressés dans `mods/<dossier-du-mod>/` ou dans le répertoire configuré. ShroudForge lit `mod.json`, le fichier facultatif `extended.mod.json`, `src/mod.lua`, `native-plugin.ini` et les DLL présentes. Un mod uniquement Lua n'a pas besoin de sa propre DLL. Un mod natif peut apparaître dans le gestionnaire de DLL sans manifeste ni fichier Lua.

Cette vue indique si le dossier et ses informations ont été reconnus. Le chargement natif réel apparaît dans le gestionnaire de DLL. La page du mod présente son état général.

## Champs inconnus du manifeste

Lorsque le module de détection et la lecture tolérante sont activés, `mod.json` et `extended.mod.json` peuvent contenir des champs inconnus de ShroudForge. Ces seuls champs ne provoquent pas le rejet du paquet. Les champs connus doivent toujours avoir des valeurs valides. Un JSON incorrect, des informations obligatoires absentes ou des types erronés restent des erreurs.

Désactiver cette compatibilité rétablit la validation stricte. Cela ne supprime ni ne désactive automatiquement tous les mods installés.

## Un mod n'est pas reconnu

Vérifiez d'abord le véritable dossier du mod. Un téléchargement peut contenir un dossier supplémentaire ou un répertoire `mods`. L'installateur recherche un seul paquet de mod. Une nouvelle archive contenant plusieurs paquets n'est pas installée automatiquement dans son ensemble. Décompressez-la et installez un dossier de mod à la fois. En fonctionnement normal, ShroudForge lit les dossiers décompressés et non les archives.

Corrigez l'emplacement JSON indiqué ou réinstallez le paquet complet. Un profil de compatibilité peut adapter des particularités connues. La reconnaissance seule ne confirme pas le bon fonctionnement des hooks natifs ou des API du mod.
