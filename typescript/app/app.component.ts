import { Component, OnInit } from '@angular/core';

import { WordService, WsWordService, MockWordService } from './word.service';
import {WordWeight} from './word-weight';

@Component({
    selector: 'my-app',
    templateUrl: '../html/app.component.html',
    providers: [WsWordService, MockWordService]
})
export class AppComponent implements OnInit {
    protected maxSize : number = 200;

    constructor(private wordService: WsWordService) { }

    ngOnInit() {
        this.setText("promessi_sposi.txt");
    }

    setText(text : string) {
        this.wordService.GetWordsCount(text).then((list) => {
            let scale = list.map((ww) => new WordWeight(ww.word, Math.pow(ww.count,2)));

            let max = scale.map((ww) => ww.count).
                reduce((max, cur) => {
                    return Math.max(max, cur);
                }, 0);

            scale = scale.map((ww) => new WordWeight(ww.word, (ww.count / max) * this.maxSize));

            let outarray = scale.map((ww) => [ww.word, ww.count]);

            WordCloud(document.getElementById("my_canvas"), {
                list: outarray,
                gridSize: 1,
                minSize: 0,
                // backgroundColor: '#333300',
                // color: function (word, weight) { return '#000000' }
            });

        });
    }
}