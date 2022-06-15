// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU General Public License version 3.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, version 3 of the License.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    The GNU General Public License is available in COPYING file
//    along with Aladin Lite.
//

import { MocServer } from "../MocServer";
import { Utils } from "../Utils";
import  autocomplete from 'autocompleter';

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/CatalogSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export class CatalogSelector {

    constructor(parentDiv, fnURLSelected, fnIdSelected) {
        this.parentDiv = parentDiv;

        this.fnURLSelected = fnURLSelected;
        this.fnIdSelected  = fnIdSelected;

        this.#createComponent();
    }

    #createComponent() {
        const self = this;

        this.mainDiv = document.createElement('div');
        this.mainDiv.classList.add('modal', 'modal-open', 'place-items-center', 'h-screen');
        this.mainDiv.style.display = 'none';

        const autocompleteId = 'autocomplete-' + Utils.uuidv4();
        this.mainDiv.insertAdjacentHTML('afterbegin', 
        '<div class="modal-box">' +
          '<label class="btn btn-sm btn-circle absolute right-2 top-2">✕</label>' +
          '<h3 class="font-bold text-lg">Select catalogue:</h3>' +
          '<br>' +
            '<div tabindex="0" class="tabs">' +
              '<a class="tab tab-lifted tab-active">By ID, title, keyword</a> ' +
              '<a class="tab tab-lifted">By URL</a>' +
            '</div>' +
            '<div>' +
            '<div class="p-4">' +
            '<input id="' + autocompleteId + '" type="text" placeholder="Type keyword" class="input input-bordered w-full max-w-xs" />' +
            '</div>' +
            '<div class="hidden p-4">' +
            '<input type="text" placeholder="Type VOTable URL" class="input input-bordered w-full max-w-xs" />' +
            '</div>' +
         ' </div>' +
            '<div class="flex space-x-2 justify-center pt-6">' +
            '<button class="btn btn-primary btn-sm">Select</button>' +
            '<button class="btn btn-outline btn-sm">Cancel</button>' +
            '</div>' +
            
        '</div>');

        this.parentDiv.appendChild(this.mainDiv);

        // setup autocomplete
        let input = document.getElementById(autocompleteId);
        
        // Query the mocserver
        MocServer.getAllCatalogHiPSes();
        autocomplete({
            input: input,
            minLength: 3,
            fetch: function(text, update) {
                text = text.toLowerCase();
                const filterCats = function(item) {
                    const ID = item.ID;
                    const obsTitle = item.obs_title || '';
                    const obsDescription = item.obs_description || '';

                    return ID.toLowerCase().includes(text) || obsTitle.toLowerCase().includes(text) || obsDescription.toLowerCase().includes(text);
                }
                // you can also use AJAX requests instead of preloaded data
                const suggestions = MocServer.getAllCatalogHiPSes().filter(filterCats);
                update(suggestions);
            },
            onSelect: function(item) {
                input.value = item.ID;
                self.selectedItem = item;
            },
            render: function(item, currentValue) {
                const itemElement = document.createElement("div");
                itemElement.innerHTML = (item.obs_title || '') + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';


                return itemElement;
            }
        });

        // tab management
        let firstTab = this.mainDiv.querySelectorAll('div div a')[0];
        let secondTab = this.mainDiv.querySelectorAll('div div a')[1];
        let firstTabContent = this.mainDiv.querySelectorAll('div div .p-4')[0];
        let secondTabContent = this.mainDiv.querySelectorAll('div div .p-4')[1];


        $(firstTab).click(function() {
            $(secondTab).removeClass('tab-active');
            $(firstTab).addClass('tab-active');
            $(secondTabContent).hide();
            $(firstTabContent).show();
        });
        $(secondTab).click(function() {
            $(firstTab).removeClass('tab-active');
            $(secondTab).addClass('tab-active');
            $(firstTabContent).hide();
            $(secondTabContent).show();
        });

        // this modal is closed when clicking on the cross at the top right, or on the Cancel button
        let closeBtn  = this.mainDiv.querySelectorAll('.btn-circle');
        let cancelBtn = this.mainDiv.querySelectorAll('.btn-outline');
        $(closeBtn).add($(cancelBtn)).click(function() {
            self.hide();
        });

        // when 'Select' is pressed, call the callbacks
        let selectBtn = this.mainDiv.querySelectorAll('.btn-primary');
        $(selectBtn).click(function() {
            let byIdSelected = $(self.mainDiv.querySelectorAll('div div a')[0]).hasClass('tab-active');

            let idInput = self.mainDiv.querySelectorAll('div div .p-4')[0].querySelector('input');
            let urlInput = self.mainDiv.querySelectorAll('div div .p-4')[1].querySelector('input');

            if (byIdSelected) {
                self.fnIdSelected && self.fnIdSelected(idInput.value, self.selectedItem, {});
            }
            else {
                self.fnURLSelected && self.fnURLSelected(urlInput.value);
            }

            idInput.value = '';
            urlInput.value = '';
        
            self.hide();

        });

    }

    show() {
        this.mainDiv.style.display = 'flex';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }

}
