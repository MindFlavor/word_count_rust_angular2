import { Injectable } from '@angular/core';
import { Http, Headers } from '@angular/http';
import 'rxjs/add/operator/toPromise';

import {WordWeight} from "./word-weight";

export interface WordService {
    GetWordsCount(name: string): Promise<WordWeight[]>;
}

@Injectable()
export class WsWordService implements WordService {
    private wordWSUrl = "http://localhost:3005";

    constructor(private http: Http) { }

    public GetWordsCount(name: string): Promise<WordWeight[]> {
        return this.http.get(this.wordWSUrl + "/" + name)
            .toPromise()
            .then((result) => {
                let wws = result.json().map((item) => new WordWeight(item[0], item[1]));

                return wws;
            });
    }
}

@Injectable()
export class MockWordService implements WordService {
    public GetWordsCount(name: string): Promise<WordWeight[]> {
        let we = undefined;
        switch (name) {
            case "alice.txt":
                we = [["alice", 409], ["regina", 80], ["voce", 74], ["re", 68], ["testuggine", 56], ["cappellaio", 56], ["falsa", 54], ["grifone", 51], ["via", 51], ["coniglio", 50], ["capo", 47], ["ora", 47], ["sorcio", 44], ["tutto", 43], ["duchessa", 43], ["illustrazione", 42], ["ghiro", 37], ["tempo", 36], ["gridò", 33], ["tre", 33]];
                break;
            case "boccaccio_decameron.txt":
                we = [["donna", 1684], ["uomo", 780], ["fatto", 626], ["casa", 542], ["giovane", 483], ["messer", 455], ["ora", 429], ["tutto", 413], ["re", 399], ["moglie", 375], ["novella", 371], ["tempo", 366], ["parole", 337], ["marito", 330], ["stato", 326], ["parte", 326], ["notte", 303], ["appresso", 298], ["amore", 284], ["volte", 280]];
                break;
            case "cecco_angiolieri_rime.txt":
                we = [["amor", 49], ["tutto", 39], ["cor", 34], ["donna", 31], ["dio", 29], ["amore", 28], ["uom", 27], ["becchina", 24], ["mal", 23], ["sed", 23], ["cecco", 21], ["vita", 20], ["par", 20], ["posso", 19], ["cuor", 19], ["mondo", 19], ["eo", 19], ["dico", 19], ["om", 18], ["anzi", 18]];
                break;
            case "divina_commedia.txt":
                we = [["occhi", 213], ["tutto", 175], ["mondo", 143], ["terra", 137], ["canto", 132], ["dio", 127], ["gente", 125], ["parte", 118], ["maestro", 111], ["colui", 108], ["sol", 106], ["ciel", 106], ["veder", 105], ["mente", 103], ["donna", 95], ["viso", 92], ["amor", 90], ["loco", 90], ["duca", 89], ["dolce", 87]];
                break;
            case "promessi_sposi.txt":
                we = [["renzo", 585], ["tutto", 520], ["ora", 450], ["don", 448], ["uomo", 424], ["lucia", 397], ["fatto", 385], ["parte", 384], ["tempo", 372], ["casa", 327], ["padre", 285], ["stato", 262], ["strada", 260], ["mano", 255], ["donna", 251], ["parole", 245], ["giorno", 240], ["abbondio", 234], ["momento", 226], ["agnese", 219]];
                break;
            case "verga_i_malavoglia.txt":
                we = [["ntoni", 543], ["don", 429], ["casa", 347], ["padron", 321], ["ora", 276], ["diceva", 262], ["dei", 216], ["colla", 209], ["mena", 187], ["michele", 184], ["compare", 183], ["nulla", 179], ["zio", 179], ["malavoglia", 177], ["occhi", 165], ["piedipapera", 163], ["colle", 156], ["tutto", 148], ["comare", 146], ["nonno", 144]];
                break;
        }
        if (we === undefined) {
            return new Promise((res, rej) => {
                rej("not found");
            });
        }
        else {
           let wws = we.map((item) => new WordWeight(item[0], item[1]));

            return new Promise((res, rej) => {
                res(wws);
            });
        }
    }
}