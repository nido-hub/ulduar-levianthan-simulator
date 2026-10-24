# Ressources graphiques — WoW local et secours originaux

## Ressources WoW copiées

`wow/{demolisher,siege,moto,leviathan}.glb` contient les géométries, textures PNG, squelettes et animations **Idle / Drive** convertis du client local 3.3.5a. Les modèles sont orientés vers −Z avec Y vertical ; les dimensions du démolisseur et du siège sont adaptées à la simulation. Chaque GLB est autonome, sans URI de fichier externe.

`wow/ui/` contient les cadres de véhicule/cible WotLK, la police Friz Quadrata et les icônes de sorts issus de `SpellIcon.dbc`. Quelques cadres sont recadrés à partir des atlas d'origine pour la barre de compétences. `wow/manifest.json` conserve la provenance et les empreintes SHA-256 des entrées/sorties. **Ces ressources restent la propriété de leurs ayants droit et ne sont pas librement redistribuables.**

Importer de nouveau, avant construction seulement :

```powershell
.\Importer-WoW.ps1 -Client 'C:\chemin\WoW' -StormLib 'C:\outils\StormLib.dll'
.\Construire.ps1
```

Prérequis d'import : Python 3, Pillow et StormLib Unicode 64 bits. La DLL de StormLib est disponible auprès de son projet officiel, https://github.com/ladislav-zezula/StormLib/releases ; elle n'est pas incluse dans le jeu. Le convertisseur lit les MPQ sans les modifier, applique la priorité des patches et n'importe que les ressources nécessaires. Il cible les quatre modèles de ce projet, pas tous les formats/modèles WoW. Le rendu n'émule pas les réflexions propriétaires, les particules ni les accessoires de combat attachés.

**Aucun import au lancement.** `Construire.ps1` copie les ressources converties dans `dist/assets/wow`. Pour déplacer le logiciel, prendre le binaire, `assets/` et `settings.toml` ; ni WoW, ni Python, ni StormLib, ni réseau ne sont requis. La salle et les effets du simulateur restent procéduraux.

## Modèle original de secours

`models/demolisher.glb` est un modèle glTF 2.0 binaire autonome. Il contient la géométrie, les matériaux et les images PNG du véhicule. Il n'a besoin d'aucune texture externe ni de connexion réseau.

## Création et modification

- Géométrie de base : `crates/client/src/scene.rs`, méthode `demolisher` et ses composants.
- Recettes de textures : `crates/client/src/textures.rs`.
- Export glTF : `crates/client/src/models.rs`.
- Régénération : `Generer-Assets.ps1`, puis `Construire.ps1`.

La géométrie de cette première étape est générée en Rust, **pas créée dans Blender**. Le GLB peut être importé dans Blender pour être retravaillé, puis exporté en glTF binaire à cet emplacement. Blender n'a pas été installé par cet essai.

Conventions à conserver :

- unités et proportions du fichier livré ; ne pas appliquer un redimensionnement global à l'import sans le reporter ;
- axe Y vertical ; le modèle exporté inclut son orientation vers −Z ;
- origine au sol sous le centre du véhicule ; pas de translation vers son point d'apparition ;
- quatre pivots nommés `Wheel:1.35` pour les roues arrière et `Wheel:1.2` pour les roues avant ; la rotation locale autour de Z anime les roues ;
- préserver les pivots, leur orientation et les noms lors de l'export Blender ;
- exporter les matériaux métal/roughness avec les textures intégrées dans le GLB.

Le chargement est asynchrone. Pendant le chargement ou en cas d'échec, le client garde le modèle procédural texturé en secours. Une fois le modèle prêt, ses pivots sont associés aux animations et le secours est masqué. F4 affiche la source actuellement utilisée.

## Direction visuelle

Les matériaux cherchent une lecture de type WotLK, et non un rendu photoréaliste : grain du bois accentué, arêtes éclaircies, fissures dessinées, ombres peintes dans les joints, fer désaturé, bronze et oxydation contenus, pierre gris-violet et reliefs géométriques. La rugosité élevée et une faible métallisation limitent l'aspect plastique/chrome.

Toutes les textures de ce **secours original** sont générées par les recettes locales, y compris celles incorporées à `models/demolisher.glb`. Cette description ne concerne pas les ressources Blizzard du dossier `wow/`.

Le pavage et les murs sont encore générés côté Rust. Ils ne font pas partie du GLB du véhicule. Le modèle reste construit à partir de primitives : une vraie passe de sculpture/modélisation dans Blender est encore nécessaire pour approcher davantage la silhouette et les détails du jeu.
