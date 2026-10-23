# Gestionnaire de DLL

Consultez les DLL de mods trouvées par ShroudForge et leur état de chargement. Les mods connus ont des profils préparés. Pour les autres DLL, sélectionnez-les et confirmez-les une fois dans le gestionnaire de DLL.

## Que faire avec un mod téléchargé ?

- **Aucune DLL fournie :** Installez le mod normalement. Les mods conçus pour ShroudForge ou EML n'ont pas besoin d'une DLL native supplémentaire.
- **Une DLL est fournie :** Placez-la dans le dossier du mod, comme indiqué ci-dessous. Elle apparaîtra ensuite dans le gestionnaire de DLL. Un profil connu a déjà préparé son choix. Sinon, sélectionnez la DLL et confirmez-la dans le gestionnaire.
- **D'autres fichiers sont fournis :** Gardez le dossier complet du mod. Ne supprimez pas les fichiers comme `mod.json`, `native-plugin.ini` ou les fichiers de configuration.

## Si le mod contient plusieurs DLL

De nombreux dossiers de mods contiennent des DLL qui ne sont que des fichiers auxiliaires. Sélectionnez uniquement les DLL indiquées pour le chargement par le profil ou les instructions de l'auteur. Si plusieurs DLL du mod doivent être chargées, définissez leur ordre dans le gestionnaire de DLL.

## Installer un mod avec une DLL

Si l'installateur ShroudForge reconnaît le paquet téléchargé, installez-le avec celui-ci. Pour une installation manuelle, créez un dossier propre à chaque mod directement dans `mods/` et placez-y la DLL du mod. Si la DLL se trouve à côté du dossier du mod dans l'archive, placez-la avec le mod. Gardez son nom d'origine et les sous-dossiers fournis avec le paquet.

```text
mods/
  mon-mod/
    mod.json        (si le paquet en contient un)
    dinput8.dll     (ou la DLL du mod)
    config.ini      (si le paquet en contient un)
```

Une DLL inconnue n'est pas chargée uniquement parce qu'elle se trouve dans le dossier. Sélectionnez-la dans le gestionnaire de DLL, vérifiez le mode de chargement proposé et confirmez-la.

## Pourquoi la DLL du mod peut-elle aussi s'appeler dinput8.dll ?

Le fichier de démarrage de ShroudForge est `dinput8.dll` dans le dossier du jeu. Le fichier du mod est séparé dans `mods/<dossier-du-mod>/dinput8.dll`. ShroudForge le charge depuis ce chemin complet. Avec un profil ou une sélection adaptée dans le gestionnaire de DLL, ShroudForge redirige les appels DirectInput vers le mod. Les deux fichiers peuvent garder le même nom. Ne renommez pas la DLL du mod et ne la placez pas dans le dossier du jeu. Ne remplacez pas le fichier de démarrage de ShroudForge.

## Comprendre les profils

Un profil est un réglage préparé pour un mod connu. Il indique à ShroudForge quels fichiers charger et comment. ShroudForge l'associe à l'identifiant du mod dans `mod.json` ou, s'il n'y en a pas, au nom du dossier. Vous pouvez aussi choisir un profil dans le gestionnaire de DLL. Les profils ne modifient pas les fichiers du mod. Une sélection personnelle enregistrée remplace la sélection de fichiers du profil.

## Chaîne DirectInput existante

Un `dinput8-chain.ini` déjà présent dans le dossier du jeu utilise le format établi :

```ini
[Chain]
mods\mon-mod\dinput8.dll
mods\autre-mod\native.dll
```

Les entrées doivent désigner des fichiers dans des paquets de mods directs comme `mods/<dossier-du-mod>/`. Les DLL à la racine du jeu et les chemins hors de ces paquets sont refusés. Leur ordre est prioritaire sur les déclarations déduites automatiquement. Un même chemin résolu n'est chargé qu'une fois. La première implémentation DirectInput valide reçoit les appels transférés. Les fichiers de démarrage propres au chargeur sont ignorés. Un transfert récursif revient à la bibliothèque Windows d'origine.

## Interrupteurs, cible et redémarrage

L'interrupteur principal contrôle tout chargement natif effectué par ShroudForge, y compris les INI, profils, chaînes et enregistrements Lua. Une sélection désactivée supprime le chargement automatique pour ce dossier. Client, serveur ou les deux détermine le processus cible. Les exclusions de noms s'appliquent juste avant le chargement. Elles ne déduisent pas les fonctions d'un mod de son nom.

Les DLL chargées restent en mémoire jusqu'à la fermeture du processus. Les désactiver, modifier leur ordre ou choisir un autre profil nécessite donc de redémarrer le jeu. Plusieurs déclarations d'un même fichier ne créent pas une autre instance.

## Une erreur de chargement apparaît

Vérifiez le nom du mod, le chemin du fichier et la cause. L'erreur Windows 126 peut signaler l'absence du fichier principal ou d'une bibliothèque requise. Si la fonction exportée DirectInput manque, vérifiez le fichier sélectionné et son mode de chargement. « Chargée » confirme le chargement, pas toutes les fonctions du mod.

Un format de plugin natif inconnu, une couche Vulkan, un chargeur complet supplémentaire ou un programme externe nécessite son intégration prévue. Charger une bibliothèque ne remplace pas cette interface.

## Installer les paquets

L'installateur trouve aussi un seul paquet de mod dans les dossiers imbriqués d'une archive. Les profils connus importent les fichiers annexes explicitement désignés. Une nouvelle archive contenant plusieurs paquets n'est pas installée automatiquement dans son ensemble. Décompressez-la et installez un dossier de mod à la fois. Le fonctionnement normal n'ouvre pas d'archives et ne démarre pas de programmes externes.
