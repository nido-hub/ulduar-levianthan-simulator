# Référence combat — WotLK 3.3.5a, dix joueurs, quatre tours, équipement 215

## Sources figées

- AzerothCore révision **9fc017d9a7eab02d59bb7417eb151b373ea1d153**, consultée le 24 octobre 2026 : https://github.com/azerothcore/azerothcore-wotlk/tree/9fc017d9a7eab02d59bb7417eb151b373ea1d153
- Dans cette révision : `boss_flame_leviathan.cpp`, `spell_generic.cpp` (Gear Scaling Ulduar 65266), `Player.cpp` (GetTotalItemLevel), `ItemTemplate.h` (qualité), `SpellAuraEffects.cpp` (cumul multiplicatif des PV), SQL `creature_template`, `creature_classlevelstats`, `creature_template_model`, `script_waypoint`, `smart_scripts`.
- Descriptions/effets 3.3.5 : https://wowgaming.altervista.org/aowow/?spell=62475 (remplacer l'ID avec ceux ci-dessous). Valeurs de sort vérifiées sur cette base ; pas les valeurs rééquilibrées de Classic.
- Modèles/animations : MPQ locaux 3.3.5a en lecture seule ; archives, noms et SHA-256 dans `assets/wow/manifest.json`.

Cette référence d'émulation n'est **pas** la configuration privée du serveur ChromieCraft. Les branches actuelles d'AzerothCore ne prouvent pas chaque détail historique Blizzard. Les différences du simulateur sont listées explicitement ci-dessous.

## Profil de référence

| Élément | Calcul / valeur |
|---|---|
| Équipement | Quinze emplacements pris en compte, tous épiques, moyenne 215. Hors chemise, tabard, distance, main gauche. |
| Scaling Ulduar | `round(((15*215-2500)/500-1)*100)` = bonus 45 %, soit facteur **1,45**. Pas la formule générique 1 %/ilvl. |
| Démolisseur | basehp2(80,classe4)=12600 × HealthModifier50 ×1,45 = **913500 PV** |
| Siège | 12600 ×90 ×1,45 = **1644300 PV** |
| Moto | 12600 ×40 ×1,45 = **730800 PV** |
| Boss normal | basehp2(83,classe1)=13945 ×1650 = **23009250 PV** |
| Boss quatre tours | 23009250 ×1,4^4 = **88392334,8 PV** ; arrondi en f32 pour le moteur |
| Tourelles 10 | Deux ×64000 PV (12600 ×5,07937, arrondi du template). |

Le scaling s'applique aux capacités, pas aux dommages infligés par les ennemis. Conducteur et passager utilisent le même équipement dans ce profil fixe. Un ilvl ne détermine **pas** le DPS à pied : le bouton utilise un tireur abstrait à 3500 DPS et 22000 PV, paramètres d'entraînement affichés/documentés, pas une certification de classe à 215.

## Capacités et unités

Les valeurs variables ci-dessous utilisent leur moyenne, sans critiques aléatoires. Toutes les portées sont en yards/unités monde, délais en secondes. Le GCD appartient au poste, pas au véhicule entier.

| Sort ID | Données source |
|---|---|
| 62306 →62307,65045 | Rocher : portée10–70, GCD1 ; impact feu30000 + Flames1000, rayon20 ; Flames libère le gel. |
| 62490 →62489,68605 | Pyrite : coût5, portée10–70, GCD1, impact60000, DoT12000/s/charge, 10charges, durée10, feu/arcane. |
| 62471 | Boost démolisseur : coût25pyrites, +100% vitesse, durée20, GCD1, pas de recharge propre. |
| 62634 →62635,65044 | Mortier : portée50, recharge1, sans GCD, impact12000 + Flames20000, rayon11/10. |
| 62308 | Bélier démolisseur20000, rayon18, recharge4, GCD1,5. |
| 62345 | Bélier siège25001, coût40, rayon18, GCD1,5. |
| 62522 | Électrochoc8720 nature, cône25, interruption4, recharge10, GCD1,5. Le coût observé en jeu/guide est38 : utilisé au lieu de18% d'une réserve100 normalisée. |
| 62346,62355 | Ruée : coût40, recharge15, GCD2 ; destination/charge du client approximée par un mouvement accéléré de5s dans ce moteur plan. |
| 62358 →62357 | Canon80000, coût20, portée10–70, GCD1, rayon20 ; Flames65045 annule le gel. |
| 64677 | Bouclier : absorption **15%** des dégâts physiques/feu/givre/arcane,5s,recharge60,GCD1,5. Pas une réduction de80%. |
| 62974 | Cor7000physique,cône35,coût20,GCD1. |
| 62286,62292 | Goudron9,45s,recharge15 ; embrasement6500/s. Le boss est ralenti de50%. |
| 62299 | Moto : +100%5s,coût50,GCD1. |
| 64660 | Soins du passager :30%PV/s pendant4s,recharge60. |
| 64414,62324 | Chargement recharge30/GCD1 ; projection recharge2/GCD1, trajectoire vers destination choisie. |

## Boss et tours

Échelles visuelles : `CreatureDisplayInfo.dbc` du client local définit le Léviathan
(affichage 28875, modèle 3088) à **3**, le conteneur de pyrite aérien
(28782, modèle 3073 `UL_PyriteFuelCellLarge`) et la cellule au sol
(28783, modèle 3074 `UL_PyriteFuelCell`) à **1,5**. Les templates AzerothCore
33113/33218/33189 ont un `DisplayScale` de 1. Ces facteurs sont intégrés aux GLB ;
ils ne modifient pas les rayons d'impact ni les portées de simulation.
Le conteneur conserve son animation Idle ; la cellule au sol n'a aucune piste
osseuse animée. Les shaders et particules spécifiques WoW restent approximés.

| Mécanique | Référence |
|---|---|
| Poursuite62374 | ~30s (script31), démolisseurs/sièges occupés, pas motos. |
| Gathering Speed62375 | +5% toutes15s, max20, persistantes au changement de poursuite, retirées au shutdown. Base course du template5u/s. |
| Bélier62376 | Attaque toutes2s à portée15, +50% dégâts reçus cumulables pendant20s. Dégâts moyens de profil10261 avant vulnérabilité/tour, pas des dégâts continus au contact. |
| Décharges62396 | Canalisation10s,3000feu/s, premières20s puis20s, portée50, interruption possible. |
| Barrage62400 | Physique4000 moyen ; premières5s puis4s. |
| Napalm63666 | Incantation2s,8000feu puis4000/s8s,rayon5. Cadence du profil12s : la constante du script AzerothCore n'est pas utilisée par son IA principale ; cette cadence reste une approximation signalée. |
| Shutdown62475 | Deux tourelles détruites :20s,+50% dégâts reçus,arrêt du boss,retrait vitesse ; éjection après3s puis réinstallation des tourelles à la reprise. Les tours continuent. |
| Tours65075/65076/65077/64482 | +40%PV par tour,multiplicatif ; +50%feu/+25%physique du boss ; réduction10%feu reçu pour Freya. Les invocations ne reçoivent pas automatiquement les bonus offensifs du boss. |
| Thorim62911→62912 | Première60s,8balises,attente5–20s,300000 dans7yards,puis300000/max(distance-6,1) selon le script. Réapparition script60–120s ; profil déterministe90s. |
| Hodir62533→62297 | Deux balises à20s,suivent à vitesse de marche2,5,verrouillage puis5s avant impact150000 dans10yards ; gel60s et1%PV/s,attaque feu pour libérer. |
| Mimiron62910 | Première42s,dépôt toutes2s,150000 à l'impact dans9yards,puis10%PV/s30s. Waypoints SQL33369 traduits, pas la boucle inventée du prototype. |
| Freya62906/62907/62947 | Quatre coins,invocations vers34s puis29s ; un Ward302400PV + un Lasher64260PV par coin ; Lash65062 toutes2s. |
| Tourelles62402 | Searing Flame500 moyen,incantation0,5s,cycle d'attaque2s ; seuls les occupants sur le boss peuvent les attaquer. |

## Exceptions de simulation connues — ne pas annoncer une copie parfaite

- IA, choix de cibles et phases aléatoires déterministes ; pas une reproduction des décisions de neuf humains. Géométrie d'arène carrée216×216, sans collision de triangle ni décor WoW exact. Centre source(270,-34), translation(-270,+34) des waypoints/coins.
- Moyennes de dégâts sans armure/évitement/critiques complets. Cadence napalm et dégâts moyens de contact/plantes restent des profils : un relevé serveur est requis pour les certifier. Les ressources du siège sont représentées sur100 unités.
- Ruée de siège comme boost, pas charge physique ; quelques capacités de transport contextuelles WoW ne sont pas exposées (moto transport de pyrite, changement libre de véhicule). Le réapprovisionnement aérien a une cadence déterministe4s.
- Commande unique des tourelles puis activation automatique des dispositifs ; pas de rotation de classe ni de sélection de spécialisation. Les dommages des tourelles suivent un profil moyen ; pas de simulation des soins de classe.
- Les dégâts périodiques de pyrite et de feu sont intégrés au pas fixe ; les états/délais de fin sont conservés. Un pas de rendu ne déclenche aucun dommage.
- Visuels Bevy reconstitués, pas shaders/particules propriétaires identiques au client ; les personnages humains restent stylisés. Les GLB végétaux utilisent les skins/scales du client, avec animations Idle/Drive/Attack.

Ces exceptions doivent rester visibles dans le README et le bilan de livraison : les règles principales sont rapprochées de WotLK, mais **la reproduction complète exacte et la fidélité au serveur ChromieCraft ne sont pas certifiées**.
