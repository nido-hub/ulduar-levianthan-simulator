# Leviathan Rust — combat natif 0.3, HM 10 / ilvl 215

Application Windows **Rust / Bevy 0.20.0** pour s'entraîner au Léviathan des flammes. Profil par défaut : **10 joueurs, quatre tours, équipement épique ilvl 215**, vitesse **7 u/s**, retour automatique de caméra **Non** (réactivable dans Échap). La conduite reste sans strafe ; la simulation de combat est indépendante du rendu.

Les modèles et textures des **démolisseurs, sièges, motos et du Léviathan** sont désormais copiés du client local et convertis en GLB autonomes, avec squelette et animations de repos/conduite. Le HUD utilise les cadres, icônes de sorts et police du client WotLK, avec PV, ressource, charges de pyrite et recharges. **WoW n'est pas requis pour lancer la livraison : conserver simplement `assets/` à côté de l'exécutable.**

La pyrite utilise aussi les vrais modèles du client : conteneur aérien animé et cellule au sol, à leur échelle d'affichage **×1,5**. Le Léviathan utilise désormais son échelle WotLK **×3**, au lieu du modèle brut ×1 ; les portées et dégâts ne changent pas.

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

En conducteur de démolisseur, le passager IA utilise son accélération quand le véhicule est poursuivi : **25 pyrites, +100 % de vitesse pendant 20 s**, uniquement s'il est encore à son poste. Il prépare ensuite son chargement ; la projection manuelle devient disponible. Un tir mal placé peut déposer le passager au sol sans atteindre le boss.

En passager de démolisseur, **5** charge la catapulte et le conducteur IA déclenche la projection à portée. Une fois sur le boss, **1 ou le bouton « Attaquer les tourelles »** lance l'attaque continue : pas besoin de marteler. Un seul tireur peut détruire successivement les deux tourelles. Leurs PV sont affichés ; le joueur ne DPS pas sans cette commande. Les deux destructions déclenchent automatiquement la surcharge : **20 s, dégâts reçus +50 %, vitesse accumulée supprimée**, puis éjection et récupération par moto. La collecte et les autres compétences de véhicule sont indisponibles hors du poste.

En moto, approcher un joueur au sol et utiliser **4**, le soigner avec **5**, puis rejoindre son démolisseur d'origine et réutiliser **4** pour le déposer. Si son démolisseur a été détruit, **4** le dépose au sol ; la destruction du porteur le libère également. Un passager vivant au sol peut se déplacer avec les commandes habituelles, mais pas pendant le vol, sur le boss ou pendant le transport. Les motos IA accomplissent les secours lorsqu'elles ne sont pas contrôlées par le joueur.

## Combat et raid simulé

- Six véhicules : deux démolisseurs, deux sièges et deux motos ; quatre passagers, soit dix occupants simulés avant les projections.
- Boss : environ **23,01 M PV normal / 88,39 M PV quatre tours** ; véhicules à **913 500 / 1 644 300 / 730 800 PV** (démolisseur/siège/moto) avec le profil épique 215.
- Poursuite des démolisseurs/sièges, charges de vitesse persistantes entre poursuites, bélier et vulnérabilité cumulable, Décharges canalisées interruptibles, barrages et napalm.
- Pyrite jusqu'à dix charges, expiration, ressources limitées et caisses aériennes/au sol.
- Goudron ralentissant le boss et les adds ; les tirs de feu peuvent l'enflammer ou libérer les véhicules gelés.
- Tours : balises/rayons de Thorim avec frappe et atténuation à distance, trajet Mimiron issu des waypoints de référence, balises de givre d'Hodir avec verrouillage et impact, gardiens et flagellants de Freya dans les quatre angles. Les plantes utilisent des modèles locaux animés (repos/conduite/attaque), avec secours végétal si l'asset manque.
- Victoire à la mort du boss, défaite à la destruction du véhicule assigné, bilan dans le HUD et nouvelle tentative rapide.

## Limites de fidélité

Les valeurs principales sont rapprochées de la référence **WotLK 3.3.5a / AzerothCore**, avec bonus des tours, scaling du profil 215 et GCD séparés par poste. Sources, formules et différences sont dans [`docs/wotlk-reference.md`](docs/wotlk-reference.md). **Ce n'est pas encore une reproduction complète exacte**, ni une certification de la configuration privée ChromieCraft : cadence du napalm, dégâts moyens de contact/plantes, ruée du siège représentée par un boost, physique et décisions IA restent des approximations explicites.

La salle reste stylisée, agrandie pour les portées du combat. La physique est horizontale, les impacts utilisent des zones ; pas de collisions entre véhicules ni de physique verticale complète. Le bouton de tourelles utilise un **profil de tireur abstrait : 3 500 DPS / 22 000 PV**, pas une rotation de classe déduite de l'ilvl. Pas de sélection de classes, de changement libre de véhicule, de transport de pyrite en moto ni de multijoueur. Les visuels procéduraux restent disponibles si les GLB sont absents ou invalides ; un chargement asynchrone défaillant ne masque pas son secours.

Le rendu conserve les textures diffuses, l'émission et les animations de repos/conduite, mais pas les shaders de réflexion, émetteurs de particules, accessoires de combat attachés ni toute la chorégraphie du client WoW. Les pilotes ne sont pas des personnages WoW importés. Le résultat n'est donc pas une reproduction pixel pour pixel du jeu.

L'IA est heuristique et ne garantit ni esquive parfaite, ni survie du raid. Elle utilise les mêmes coûts/recharges et une visée simplifiée vers le boss ; le joueur vise ses projectiles manuellement. Les gunners font tomber les caisses avant collecte. Les moyennes de dégâts et les choix déterministes remplacent le hasard, l'armure et les rotations de dix humains. Les phases et modèles doivent encore être confrontés à des relevés du serveur pour annoncer une fidélité intégrale.

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

### Validation du profil HM 215 — 24 octobre 2026

- **99 tests Rust passent** : 20 conduite/caméra, 53 combat et 26 configuration/visée/assets/HUD. **14 tests Python** passent avec accès MPQ réel (`WOW_STORMLIB`) ; sinon ce seul test est ignoré. Régressions : boost IA payé, GCD indépendants/hors GCD, bouton tourelles, dégâts au sol, vol/transport, santé absolue et changement de difficulté.
- Les cinq postes ont passé des essais GPU de plus de 20 secondes ; dix scènes et dix lecteurs d'animation chargent leurs ressources copiées. Captures et rapports `dist/validation-hm215-<poste>.*`. L'essai **passager de 101 secondes** a été rejoué après le dernier correctif de ciblage des plantes : une surcharge, 24 scènes végétales chargées. Les quatre essais courts précèdent ce dernier correctif, dont le chemin de code n'est atteint qu'après les premières vagues de Freya ; pas de certification exhaustive des cinq postes.
- Le menu et le secours procédural ont passé leur essai natif après ce dernier correctif. Les anciennes validations `validation-wow-*`, `validation-final-*` et autres captures antérieures sont historiques.
- **Autonomie vérifiée** : copie du binaire, des assets et des réglages hors de WoW, lancement depuis `C:\Windows`, essai à **960 × 640** ; rapport `dist/validation-hm215-standalone.txt` et `.png`, provenant des assets copiés. Ce contrôle utilise le scénario natif automatisé, pas des clics humains exhaustifs.
- Les réglages de la livraison ont été comparés avant/après le test autonome et préservés. Les tests de rendu maintiennent la santé du joueur et du boss pour atteindre les phases ; un scénario headless distinct teste la fin d'une tentative HM sans ces aides.

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
