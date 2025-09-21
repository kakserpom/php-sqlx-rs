<?php
include 'pdo.php';
$pdo = new PdoEmulator\Pdo('pgsql:dbname=postgres;host:localhost');
$stmt = $pdo->query("SELECT 123");
foreach ($stmt as $row) {
    var_dump($stmt);
}