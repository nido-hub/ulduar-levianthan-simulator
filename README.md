# Leviathan Rust — combat natif 0.3, visuels WoW locaux

Prototype Windows **Rust / Bevy 0.20.0** pour s'entraîner au Léviathan des flammes. La conduite et la caméra de la version validée sont conservées, avec suppression du strafe, et la simulation de combat tourne dans un module indépendant du rendu.

Les modèles et textures des **démolisseurs, sièges, motos et du Léviathan** sont désormais copiés du client local et convertis en GLB autonomes, avec squelette et animations de repos/conduite. Le HUD utilise les cadres, icônes de sorts et police du client WotLK, avec PV, ressource, charges de pyrite et recharges. **WoW n'est pas requis pour lancer la livraison : conserver simplement `assets/` à côté de l'exécutable.**

## Télécharger et jouer sans compiler

Sur GitHub : **Code → Download ZIP**, puis extraire **toute l'archive** et double-cliquer sur **`Lancer.cmd`** dans le dossier extrait. Ne pas lancer le jeu directement depuis l'archive et ne pas déplacer l'exécutable sans ses ressources.

Le dépôt inclut `dist/LeviathanRust-0.3.exe`, `dist/assets/` et `dist/settings.toml` : aucune installation de Rust, de WoW ou d'outil d'import n'est nécessaire pour jouer sous Windows x64. Une carte graphique et des pilotes compatibles restent nécessaires ; le runtime Microsoft Visual C++ peut être requis. Cette livraison Windows n'est pas une version jouable dans le navigateur.

Les ressources Blizzard copiées sont incluses à la demande de l'auteur du dépôt ; cette présence ne constitue pas une licence libre ni une autorisation de redistribution.

## Lancer la bonne version

Double-cliquer sur **`Lancer.cmd`**, ou sur **`dist/LeviathanRust-0.3.exe`**. Le binaire est versionné pour pouvoir le livrer sans écraser l'ancienne application encore ouverte. **`dist/LeviathanRust.exe` est l'ancienne version : elle n'est plus la cible du lanceur.**

Garder `dist/settings.toml` et `dist/assets/` à côté du nouvel exécutable. Une fois construit, il ne nécessite ni Rust, ni Blender, ni navigateur, ni serveur, ni connexion à WoW. Un GPU compatible Bevy/wgpu et un pilote à jour sont nécessaires ; une autre machine peut demander le runtime Microsoft Visual C++.

## Premier combat

1. **F2** : choisir son poste avant le pull. Les autres postes sont contrôlés par l'IA.
2. **F7** : quatre tours actives ou désactivées. Quatre tours par défaut.
3. **Entrée** : lancer le combat.
4. **1–5**, rangée du haut ou pavé numérique : sélectionner un sort ; les boutons de la barre sont également cliquables.
5. Pour les projectiles, **viser le point d'impact puis cliquer à gauche**. Le cercle vert indique une portée valide ; rouge signifie hors portée. **Clic droit ou Échap** annule la visée.
6. **F5** remet toute la tentative à zéro : boss, raid, ressources, effets et bilan. Le rôle est conservé.

Le HUD montre les PV du boss et du véhicule, la ressource du poste, la poursuite, les Décharges de flammes, les charges de pyrite, les recharges, les tirs touchés/manqués, les interruptions et les secours. Les noms au-dessus des unités permettent de distinguer les véhicules. Les gel et bouclier ont un effet visuel.

## Conduite corrigée : pas de glissement latéral

| Action | Commande |
|---|---|
| Avancer / reculer | Z / S sur AZERTY, W / S sur QWERTY, flèches haut / bas |
| Tourner | Q / D sur AZERTY, A / D sur QWERTY, flèches gauche / droite |
| Regarder autour | Clic gauche maintenu, hors du mode de visée |
| Orienter véhicule et caméra | Clic droit maintenu, hors du mode de visée |
| Avancer à la souris | Boutons gauche et droit maintenus, hors du mode de visée |
| Zoom / recentrage | Molette / Début (Home) |
| Replacer avant le pull | R |
| Pause et réglages | Échap, ou bouton du menu |
| Aide / mesures | F1 / F4 |
| Capture native PNG | F12 |

**Le clic droit ne transforme plus les touches de rotation en déplacement latéral. A/E (Q/E en QWERTY) ne déplacent plus les véhicules de côté.** La réponse adoucie agit uniquement sur la vitesse longitudinale : elle ne conserve pas d'inertie latérale après une rotation à la souris. Relâcher l'avance arrête le déplacement, y compris dans ce mode.

Un passager ne conduit pas : le conducteur IA déplace son véhicule. Le déplacement est également bloqué pendant un gel, après destruction ou en fin de tentative. La perte du focus met le jeu en pause et libère le curseur ; elle annule aussi les tirs en attente.

## Cinq postes jouables

| Poste | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| Démolisseur conducteur | Rocher | Pyrite | Bélier | Projeter passager | — |
| Démolisseur passager | Mortier | Roquette anti-air | Saisir caisse | Accélération | Charger catapulte |
| Siège conducteur | Bélier | Électrochoc | Ruée vapeur | — | — |
| Siège tourelle | Roquette anti-air | Canon | Bouclier | — | — |
| Moto | Cor sonore | Goudron | Accélération | Secours / déposer | Premiers soins |

### Visée et projectiles

- Aucun verrouillage automatique sur le boss. Le point choisi reste fixe après le tir : **il faut anticiper son déplacement pendant le vol**.
- Rocher, pyrite, mortier et canon suivent une trajectoire vers le point visé au sol. Le contact est résolu par une zone d'impact à l'arrivée, pas par une collision sur chaque triangle du modèle.
- Les roquettes anti-air visent le **plan des caisses aériennes**, pas le sol ; une roquette réussie fait tomber la caisse. Elles ne servent pas à infliger des dégâts au boss.
- Le crochet doit viser une caisse déjà au sol et dans sa portée. Il récupère une caisse, pas toute une pile.
- Bélier, Électrochoc et cor sonore utilisent un cône devant le véhicule. Un boss derrière le siège ne peut pas être interrompu par l'Électrochoc.
- Les coûts et recharges ne sont pas consommés pour une visée invalide ou des préconditions manquantes. Les recharges et réserves du conducteur et du tireur de siège sont séparées.

### Projections et secours

En conducteur de démolisseur, le passager IA prépare son chargement après un délai ; la projection manuelle est ensuite disponible. Un tir mal placé peut déposer le passager au sol sans atteindre le boss.

En passager de démolisseur, **5** charge la catapulte et le conducteur IA déclenche la projection à portée. La collecte est indisponible pendant l'absence du passager. Une fois deux passagers sur le boss, la neutralisation des tourelles est **automatisée** ; la surcharge immobilise le boss, augmente les dégâts reçus et ramène les passagers au sol.

En moto, approcher un joueur au sol et utiliser **4**, le soigner avec **5**, puis rejoindre son démolisseur d'origine et réutiliser **4** pour le déposer. Les motos IA accomplissent les secours lorsqu'elles ne sont pas contrôlées par le joueur.

## Combat et raid simulé

- Six véhicules : deux démolisseurs, deux sièges et deux motos ; quatre passagers, soit dix occupants simulés avant les projections.
- Boss mobile, poursuite des démolisseurs/sièges, changement de cible, augmentation progressive de vitesse, percussion de proximité, Décharges interruptibles et dégâts de barrage simplifiés.
- Pyrite jusqu'à dix charges, expiration, ressources limitées et caisses aériennes/au sol.
- Goudron ralentissant le boss et les adds ; les tirs de feu peuvent l'enflammer ou libérer les véhicules gelés.
- Tours : impacts de Thorim, boucle centrale horaire en losange arrondi pour Mimiron, rayon de givre d'Hodir, vagues de renforts de Freya.
- Victoire à la mort du boss, défaite à la destruction du véhicule assigné, bilan dans le HUD et nouvelle tentative rapide.

## Limites de fidélité

C'est une **première implémentation jouable**, pas une certification du combat original 3.3.5. PV, dégâts, portées, ressources, recharges, courbes de vitesse et calendriers sont normalisés pour l'entraînement. Les bonus passifs exacts des tours, le GCD complet et le scaling d'équipement ne sont pas reproduits.

La salle reste une création originale stylisée. Les véhicules sont les modèles WoW copiés, avec des dimensions adaptées à la simulation existante. La physique reste sur un plan horizontal ; les trajectoires sont visuelles et les impacts utilisent des zones simplifiées. Pas de collisions entre véhicules ni de physique verticale complète, pas de sélection de classes ou combat manuel sur les tourelles, pas de multijoueur. Les visuels originaux restent disponibles en secours si les GLB sont absents ou invalides ; un chargement asynchrone défaillant ne masque pas son secours.

Le rendu conserve les textures diffuses, l'émission et les animations de repos/conduite, mais pas les shaders de réflexion, émetteurs de particules, accessoires de combat attachés ni toute la chorégraphie du client WoW. Les pilotes ne sont pas des personnages WoW importés. Le résultat n'est donc pas une reproduction pixel pour pixel du jeu.

L'IA est heuristique et ne garantit ni esquive parfaite, ni survie du raid. Elle utilise une visée simplifiée vers le boss. Le joueur, lui, ne bénéficie pas de cette automatisation pour ses projectiles. Les gunners alliés peuvent faire tomber des caisses ; un ravitaillement manuel rapide reste possible. Les vagues de Freya et les effets élémentaires sont des représentations simplifiées. La phase des tourelles est une séquence automatique après les deux projections, pas un nouveau poste de DPS à pied.

## Réglages et ressources graphiques

Le menu Échap conserve vitesse, rotation, sensibilité, champ de vision, réponse directe/adoucie, retour caméra, inversion verticale, ombres et VSync. Les changements restent en mémoire jusqu'à **Enregistrer**. Le script de construction ne remplace pas `dist/settings.toml`. Les champs de strafe des anciennes configurations sont conservés pour compatibilité mais ignorés pour les véhicules.

Les noms des touches de `[bindings]` désignent les **positions physiques QWERTY** : `W` correspond à Z et `A` à Q sur AZERTY. Les flèches restent actives. Les touches globales et celles des compétences ne sont pas remappables dans cette version.

Les textures originales et le pipeline GLB de la 0.2 sont conservés en secours. Voir [`assets/README.md`](assets/README.md). `Generer-Assets.ps1` régénère uniquement l'ancien démolisseur original : conserver une copie avant de l'exécuter sur un modèle retravaillé dans Blender ; il ne remplace pas `assets/wow/`.

Les ressources WoW sont extraites **uniquement lors de l'import**, depuis les MPQ ouverts en lecture seule. `Importer-WoW.ps1 -Client <dossier-WoW> -StormLib <DLL>` permet de les régénérer avec Python 3, Pillow et la DLL Unicode 64 bits de StormLib. Ces outils ne sont pas nécessaires pour jouer. `assets/wow/manifest.json` conserve les noms internes, archives et empreintes des ressources copiées, sans chemin absolu vers le client. Les ressources Blizzard ne sont pas librement redistribuables : ne pas publier cette livraison avec ses ressources sans les autorisations nécessaires.

## Construire et tester

Prérequis de compilation : Rust stable compatible (minimum 1.97.1), outils C++ MSVC et Windows SDK. Bevy est figé à `=0.20.0`, les dépendances sont figées dans `Cargo.lock`. Le binaire est un build de développement optimisé, sans liaison dynamique Bevy.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\Construire.ps1
cargo test --workspace --locked
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Menu
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role demo
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role demo-passenger
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role siege
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role gunner
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role moto
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tester.ps1 -Combat -Role demo -Seconds 75
python -m unittest discover -s tools -v
```

Les tests de rendu de combat conservent volontairement le joueur automatisé en vie ; les tests longs maintiennent aussi les PV du boss pour atteindre les mécaniques tardives. **Ces aides existent uniquement avec les arguments de validation, jamais dans une partie normale.** Ces scénarios ne sont pas des benchmarks de survie ou une preuve de la justesse de l'équilibrage. Le test natif vérifie également la projection écran/monde et le chargement du GLB avec ses quatre pivots de roues.

Les captures et rapports sont dans `dist/validation*.png` et `dist/validation*.txt`. Les logs du dernier test sont dans `dist/native-stderr.log`. En cas de plantage, regarder `dist/panic.log`. Le script vérifie la fraîcheur des rapports ; aucun résultat ancien n'est utilisé pour déclarer un succès.

### Validation des nouveaux visuels — 24 octobre 2026

- **70 tests Rust passent** : 20 conduite/caméra, 27 combat et 23 configuration/visée/assets/HUD. Treize tests Python du convertisseur passent, dont l'accès MPQ réel lorsque `WOW_STORMLIB` désigne la DLL locale (sinon ce seul test est ignoré). Les contrôles incluent les valeurs non finies et les animations obligatoires manquantes ou de durée nulle.
- Les cinq postes ont chacun passé un essai GPU de plus de 20 secondes ; dix scènes et dix lecteurs d'animation chargent leurs ressources copiées. Captures par poste dans `dist/validation-combat-<poste>.png`.
- Le menu et le secours procédural ont chacun passé leur essai natif. Un essai passager de **76 secondes** a aussi passé la validation, avec une surcharge du boss et maintien des dix scènes/lecteurs d'animation ; rapport `dist/validation-wow-passenger-long.txt`. Les anciennes validations prolongées du conducteur restent historiques et n'ont pas été rejouées pour ce changement visuel.
- **Autonomie vérifiée** : copie du binaire, des assets et des réglages dans un répertoire temporaire hors de WoW, lancement depuis un autre répertoire, essai à 960 × 640 et activation d'une compétence via les messages souris de la fenêtre native. Le rapport confirme que les dix GLB viennent des assets copiés : `dist/validation-standalone.txt` et `.png`.
- Les réglages de la livraison existante ont été comparés avant/après le test autonome et préservés. Les captures ne remplacent pas une validation humaine exhaustive de chaque animation et de chaque condition de combat.

Ces tests ne valident pas une fidélité parfaite des valeurs 3.3.5, une IA infaillible ni le ressenti humain de toutes les situations. Les modèles et l'équilibrage restent ceux d'un prototype.

## Architecture

```text
crates/driving/   règles de conduite et caméra, indépendantes du GPU
crates/combat/    boss, rôles, sorts, projectiles, ressources, IA et tours
crates/client/    Bevy, rendu, visée souris, interface, modèles et textures
assets/          modèles originaux de secours et ressources WoW converties
tools/           conversion préalable MPQ/M2/SKIN/BLP et tests Python
dist/            binaire 0.3, assets, réglages et guide court
```

Le prototype web `../LeviathanTrainer` et l'ancien binaire restent distincts de cette livraison. Projet non officiel, sans affiliation Blizzard et sans interaction avec WoW. Bevy et les dépendances restent sous leurs licences respectives.
